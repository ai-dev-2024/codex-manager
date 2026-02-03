use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, Clear, Gauge, List, ListItem, Paragraph, Row, Sparkline, Table,
        Tabs, Wrap,
    },
    Frame, Terminal,
};
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    config::Config,
    models::{Account, UsageSnapshot},
    routing::{RoutingEngine, RoutingStats, RoutingStrategy},
    storage::EncryptedStore,
};

/// CLI Application
pub struct CliApp {
    store: EncryptedStore,
    routing_engine: Arc<RoutingEngine>,
    config: Config,
    accounts: Vec<Account>,
    usage_data: std::collections::HashMap<Uuid, UsageSnapshot>,
    selected_tab: usize,
    selected_account: usize,
    show_add_dialog: bool,
    show_delete_confirm: bool,
    new_account_label: String,
    new_account_key: String,
    status_message: Option<String>,
}

impl CliApp {
    pub fn new(
        store: EncryptedStore,
        routing_engine: Arc<RoutingEngine>,
        config: Config,
    ) -> Self {
        Self {
            store,
            routing_engine,
            config,
            accounts: Vec::new(),
            usage_data: std::collections::HashMap::new(),
            selected_tab: 0,
            selected_account: 0,
            show_add_dialog: false,
            show_delete_confirm: false,
            new_account_label: String::new(),
            new_account_key: String::new(),
            status_message: None,
        }
    }

    /// Run the TUI application
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Load initial data
        self.refresh_data().await;

        // Main event loop
        let mut last_update = std::time::Instant::now();
        let update_interval = Duration::from_secs(5);

        loop {
            // Draw UI
            terminal.draw(|f| self.draw(f))?;

            // Poll for events with timeout
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if self.handle_key_event(key).await? {
                        break;
                    }
                }
            }

            // Periodic refresh
            if last_update.elapsed() >= update_interval {
                self.refresh_data().await;
                last_update = std::time::Instant::now();
            }
        }

        // Restore terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    }

    /// Refresh account and usage data
    async fn refresh_data(&mut self,
    ) {
        match self.store.load_accounts() {
            Ok(accounts) => {
                self.accounts = accounts;

                // Load usage for each account
                for account in &self.accounts {
                    if let Ok(Some(usage)) = self.store.load_latest_usage(account.id) {
                        self.usage_data.insert(account.id, usage);
                    }
                }

                // Update routing engine
                self.routing_engine
                    .update_accounts(self.accounts.clone(), self.usage_data.clone())
                    .await;
            }
            Err(e) => {
                error!("Failed to load accounts: {}", e);
            }
        }
    }

    /// Handle keyboard events
    async fn handle_key_event(
        &mut self,
        key: KeyEvent,
    ) -> Result<bool> {
        // Handle dialog input first
        if self.show_add_dialog {
            return self.handle_add_dialog_key(key).await;
        }

        if self.show_delete_confirm {
            return self.handle_delete_confirm_key(key).await;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            KeyCode::Tab | KeyCode::Right => {
                self.selected_tab = (self.selected_tab + 1) % 3;
            }
            KeyCode::BackTab | KeyCode::Left => {
                self.selected_tab = (self.selected_tab + 2) % 3;
            }
            KeyCode::Char('a') => {
                self.show_add_dialog = true;
                self.new_account_label.clear();
                self.new_account_key.clear();
            }
            KeyCode::Char('d') => {
                if !self.accounts.is_empty() {
                    self.show_delete_confirm = true;
                }
            }
            KeyCode::Char('e') => {
                self.toggle_account_enabled().await?;
            }
            KeyCode::Char('r') => {
                self.refresh_data().await;
                self.status_message = Some("Data refreshed".to_string());
            }
            KeyCode::Up => {
                if self.selected_tab == 0 && !self.accounts.is_empty() {
                    self.selected_account =
                        self.selected_account.saturating_sub(1);
                }
            }
            KeyCode::Down => {
                if self.selected_tab == 0 && !self.accounts.is_empty() {
                    self.selected_account =
                        (self.selected_account + 1).min(self.accounts.len() - 1);
                }
            }
            _ => {}
        }

        Ok(false)
    }

    /// Handle keys in add account dialog
    async fn handle_add_dialog_key(
        &mut self,
        key: KeyEvent,
    ) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.show_add_dialog = false;
            }
            KeyCode::Enter => {
                if !self.new_account_label.is_empty() && !self.new_account_key.is_empty() {
                    self.add_account().await?;
                    self.show_add_dialog = false;
                }
            }
            KeyCode::Tab => {
                // Toggle between fields
            }
            KeyCode::Char(c) => {
                // Simple input handling - would need better cursor management in production
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    if c == 'c' {
                        self.show_add_dialog = false;
                    }
                } else {
                    // For simplicity, just add to both fields or track focus
                    self.new_account_label.push(c);
                }
            }
            KeyCode::Backspace => {
                self.new_account_label.pop();
            }
            _ => {}
        }

        Ok(false)
    }

    /// Handle keys in delete confirmation dialog
    async fn handle_delete_confirm_key(
        &mut self,
        key: KeyEvent,
    ) -> Result<bool> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                self.delete_account().await?;
                self.show_delete_confirm = false;
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                self.show_delete_confirm = false;
            }
            _ => {}
        }

        Ok(false)
    }

    /// Add a new account
    async fn add_account(
        &mut self,
    ) -> Result<()> {
        let account = Account::new(
            self.new_account_label.clone(),
            self.new_account_key.clone(),
        );

        self.store.save_account(&account)?;
        self.refresh_data().await;
        self.status_message = Some(format!("Added account: {}", account.label));

        info!("Added new account: {}", account.label);
        Ok(())
    }

    /// Delete selected account
    async fn delete_account(
        &mut self,
    ) -> Result<()> {
        if let Some(account) = self.accounts.get(self.selected_account) {
            let id = account.id;
            let label = account.label.clone();

            self.store.delete_account(id)?;
            self.refresh_data().await;

            // Adjust selection
            if self.selected_account >= self.accounts.len() && !self.accounts.is_empty() {
                self.selected_account = self.accounts.len() - 1;
            }

            self.status_message = Some(format!("Deleted account: {}", label));
            info!("Deleted account: {}", label);
        }

        Ok(())
    }

    /// Toggle enabled state of selected account
    async fn toggle_account_enabled(
        &mut self,
    ) -> Result<()> {
        if let Some(account) = self.accounts.get_mut(self.selected_account) {
            account.enabled = !account.enabled;
            self.store.save_account(account)?;
            self.refresh_data().await;

            let status = if account.enabled { "enabled" } else { "disabled" };
            self.status_message = Some(format!("{} {}", account.label, status));
        }

        Ok(())
    }

    /// Draw the UI
    fn draw(&self,
        f: &mut Frame,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.area());

        // Header
        self.draw_header(f, chunks[0]);

        // Main content
        match self.selected_tab {
            0 => self.draw_accounts_tab(f, chunks[1]),
            1 => self.draw_routing_tab(f, chunks[1]),
            2 => self.draw_status_tab(f, chunks[1]),
            _ => {}
        }

        // Footer
        self.draw_footer(f, chunks[2]);

        // Draw dialogs on top
        if self.show_add_dialog {
            self.draw_add_dialog(f);
        }

        if self.show_delete_confirm {
            self.draw_delete_dialog(f);
        }
    }

    /// Draw header with tabs
    fn draw_header(
        &self,
        f: &mut Frame,
        area: Rect,
    ) {
        let titles = vec!["Accounts", "Routing", "Status"];
        let tabs = Tabs::new(titles)
            .select(self.selected_tab)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .title("Codex Account Manager")
                    .borders(Borders::ALL),
            );

        f.render_widget(tabs, area);
    }

    /// Draw accounts tab
    fn draw_accounts_tab(
        &self,
        f: &mut Frame,
        area: Rect,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        // Account list
        let items: Vec<ListItem> = self
            .accounts
            .iter()
            .enumerate()
            .map(|(i, acc)| {
                let usage = self.usage_data.get(&acc.id);
                let util = usage.map(|u| u.utilization_ratio()).unwrap_or(0.0);

                let status = if acc.enabled { "●" } else { "○" };
                let style = if i == self.selected_account {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if !acc.enabled {
                    Style::default().fg(Color::Gray)
                } else {
                    Style::default()
                };

                let text = format!(
                    "{} {} (P:{}) - {:.1}%",
                    status, acc.label, acc.priority, util * 100.0
                );

                ListItem::new(text).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title("Accounts [a:add d:delete e:toggle]").borders(Borders::ALL));

        f.render_widget(list, chunks[0]);

        // Account details
        if let Some(account) = self.accounts.get(self.selected_account) {
            let usage = self.usage_data.get(&account.id);
            let details = self.format_account_details(account, usage);
            let paragraph = Paragraph::new(details)
                .block(Block::default().title("Details").borders(Borders::ALL))
                .wrap(Wrap { trim: true });

            f.render_widget(paragraph, chunks[1]);
        }
    }

    /// Draw routing tab
    fn draw_routing_tab(
        &self,
        f: &mut Frame,
        area: Rect,
    ) {
        // This would show routing statistics in a real implementation
        let text = "Routing statistics would be shown here\n\nPress 'r' to refresh data";
        let paragraph = Paragraph::new(text)
            .block(Block::default().title("Routing Engine").borders(Borders::ALL));

        f.render_widget(paragraph, area);
    }

    /// Draw status tab
    fn draw_status_tab(
        &self,
        f: &mut Frame,
        area: Rect,
    ) {
        let text = format!(
            "Proxy Server: http://{}\nAPI Key: {}\n\nTotal Accounts: {}\nPress 'r' to refresh",
            self.config.proxy.bind_addr,
            self.config.proxy.api_key,
            self.accounts.len()
        );

        let paragraph = Paragraph::new(text)
            .block(Block::default().title("System Status").borders(Borders::ALL));

        f.render_widget(paragraph, area);
    }

    /// Draw footer with help and status
    fn draw_footer(
        &self,
        f: &mut Frame,
        area: Rect,
    ) {
        let help_text =
            "q:Quit | Tab:Next Tab | ↑↓:Navigate | a:Add | d:Delete | e:Toggle | r:Refresh";

        let text = if let Some(status) = &self.status_message {
            format!("{} | Status: {}", help_text, status)
        } else {
            help_text.to_string()
        };

        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::TOP));

        f.render_widget(paragraph, area);
    }

    /// Draw add account dialog
    fn draw_add_dialog(
        &self,
        f: &mut Frame,
    ) {
        let area = centered_rect(60, 40, f.area());

        let block = Block::default()
            .title("Add Account")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));

        let text = format!(
            "Label: {}\n\nAPI Key: {}\n\n[Enter] Save  [Esc] Cancel",
            self.new_account_label,
            "*".repeat(self.new_account_key.len())
        );

        let paragraph = Paragraph::new(text).block(block);

        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }

    /// Draw delete confirmation dialog
    fn draw_delete_dialog(
        &self,
        f: &mut Frame,
    ) {
        let area = centered_rect(50, 20, f.area());

        let account_label = self
            .accounts
            .get(self.selected_account)
            .map(|a| a.label.as_str())
            .unwrap_or("Unknown");

        let block = Block::default()
            .title("Confirm Delete")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));

        let text = format!(
            "Delete account '{}' ?\n\n[y] Yes  [n] No",
            account_label
        );

        let paragraph = Paragraph::new(text).block(block);

        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }

    /// Format account details for display
    fn format_account_details(
        &self,
        account: &Account,
        usage: Option<&UsageSnapshot>,
    ) -> String {
        let mut lines = vec![
            format!("ID: {}", account.id),
            format!("Label: {}", account.label),
            format!("Priority: {}", account.priority),
            format!("Enabled: {}", account.enabled),
            String::new(),
        ];

        if let Some(org) = &account.org_id {
            lines.push(format!("Org ID: {}", org));
        }

        if let Some(daily) = account.daily_limit {
            lines.push(format!("Daily Limit: ${:.2}", daily));
        }

        if let Some(monthly) = account.monthly_limit {
            lines.push(format!("Monthly Limit: ${:.2}", monthly));
        }

        if let Some(u) = usage {
            lines.push(String::new());
            lines.push("Usage:".to_string());
            lines.push(format!("  Tokens: {}", u.tokens_used));
            lines.push(format!("  Cost: ${:.4}", u.cost_estimate));
            lines.push(format!("  Monthly: ${:.2}", u.monthly_usage));
            if let Some(remaining) = u.remaining_budget {
                lines.push(format!("  Remaining: ${:.2}", remaining));
            }
            lines.push(format!("  Utilization: {:.1}%", u.utilization_ratio() * 100.0));
        }

        lines.join("\n")
    }
}

/// Helper function to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
