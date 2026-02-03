import { Github, ExternalLink, Heart, Command } from 'lucide-react';

export default function About() {
    const version = '1.0.0';

    return (
        <div className="max-w-2xl mx-auto"
        
            {/* Logo */}
            <div className="text-center mb-8"
                <div className="inline-flex items-center justify-center w-24 h-24 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-3xl shadow-xl mb-6"
                    <Command className="w-12 h-12 text-white" />
                </div>
                <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2"
                    Codex Manager
                </h1>
                <p className="text-gray-500 dark:text-gray-400"
                    Manage your OpenAI API accounts with ease
                </p>
                <div className="mt-4 inline-flex items-center gap-2 px-3 py-1 bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400 rounded-full text-sm font-medium"
                    v{version}
                </div>
            </div>

            {/* Description */}
            <div className="bg-white dark:bg-gray-900 rounded-2xl border border-gray-200 dark:border-gray-800 p-6 mb-6"
                <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-3"
                    About
                </h2>
                <p className="text-gray-600 dark:text-gray-400 leading-relaxed"
                    Codex Manager is a powerful desktop application for managing multiple OpenAI API accounts. 
                    It provides an intuitive interface for account switching, quota monitoring, and proxy configuration.
                </p>
            </div>

            {/* Features */}
            <div className="bg-white dark:bg-gray-900 rounded-2xl border border-gray-200 dark:border-gray-800 p-6 mb-6"
                <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4"
                    Features
                </h2>
                <ul className="space-y-2 text-gray-600 dark:text-gray-400"
                    <li className="flex items-center gap-2"
                        <span className="w-1.5 h-1.5 bg-blue-500 rounded-full"></span>
                        Multi-account management with easy switching
                    </li>
                    <li className="flex items-center gap-2"
                        <span className="w-1.5 h-1.5 bg-blue-500 rounded-full"></span>
                        Real-time quota monitoring and alerts
                    </li>
                    <li className="flex items-center gap-2"
                        <span className="w-1.5 h-1.5 bg-blue-500 rounded-full"></span>
                        Built-in API proxy with load balancing
                    </li>
                    <li className="flex items-center gap-2"
                        <span className="w-1.5 h-1.5 bg-blue-500 rounded-full"></span>
                        Import/export account configurations
                    </li>
                    <li className="flex items-center gap-2"
                        <span className="w-1.5 h-1.5 bg-blue-500 rounded-full"></span>
                        Dark/light mode support
                    </li>
                </ul>
            </div>

            {/* Tech Stack */}
            <div className="bg-white dark:bg-gray-900 rounded-2xl border border-gray-200 dark:border-gray-800 p-6 mb-6"
                <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4"
                    Built With
                </h2>
                <div className="flex flex-wrap gap-2"
                    <span className="px-3 py-1 bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg text-sm"
                        Tauri v2
                    </span>
                    <span className="px-3 py-1 bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg text-sm"
                        React 18
                    </span>
                    <span className="px-3 py-1 bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg text-sm"
                        TypeScript
                    </span>
                    <span className="px-3 py-1 bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg text-sm"
                        Tailwind CSS
                    </span>
                    <span className="px-3 py-1 bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg text-sm"
                        Zustand
                    </span>
                </div>
            </div>

            {/* Links */}
            <div className="flex gap-4"
                <a
                    href="https://github.com"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="flex-1 flex items-center justify-center gap-2 p-4 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-xl hover:border-blue-300 dark:hover:border-blue-700 transition-all"
                
                    <Github className="w-5 h-5 text-gray-600 dark:text-gray-400" />
                    <span className="text-gray-700 dark:text-gray-300 font-medium"
                        GitHub
                    </span>
                    <ExternalLink className="w-4 h-4 text-gray-400" />
                </a>
            </div>

            {/* Footer */}
            <div className="mt-8 text-center"
                <p className="text-sm text-gray-500 dark:text-gray-400 flex items-center justify-center gap-1"
                    Made with <Heart className="w-4 h-4 text-rose-500 fill-rose-500" /> for developers
                </p>
                <p className="text-xs text-gray-400 dark:text-gray-500 mt-2"
                    © 2024 Codex Manager. All rights reserved.
                </p>
            </div>
        </div>
    );
}
