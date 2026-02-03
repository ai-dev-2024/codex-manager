import { ChevronLeft, ChevronRight } from 'lucide-react';
import { cn } from '../../utils/cn';

interface PaginationProps {
    currentPage: number;
    totalPages: number;
    onPageChange: (page: number) => void;
    totalItems: number;
    itemsPerPage: number;
    onPageSizeChange?: (pageSize: number) => void;
    pageSizeOptions?: number[];
}

export default function Pagination({
    currentPage,
    totalPages,
    onPageChange,
    totalItems,
    itemsPerPage,
    onPageSizeChange,
    pageSizeOptions = [10, 20, 50, 100],
}: PaginationProps) {
    if (totalPages <= 1 && !onPageSizeChange) return null;

    // Calculate displayed page range (max 5 pages)
    let startPage = Math.max(1, currentPage - 2);
    let endPage = Math.min(totalPages, startPage + 4);

    if (endPage - startPage < 4) {
        startPage = Math.max(1, endPage - 4);
    }

    const pages = [];
    for (let i = startPage; i <= endPage; i++) {
        pages.push(i);
    }

    const startIndex = (currentPage - 1) * itemsPerPage + 1;
    const endIndex = Math.min(currentPage * itemsPerPage, totalItems);

    return (
        <div className="flex items-center justify-between px-4 py-3 bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800">
            {/* Mobile View */}
            <div className="flex flex-1 justify-between sm:hidden">
                <button
                    onClick={() => onPageChange(currentPage - 1)}
                    disabled={currentPage === 1}
                    className={cn(
                        'relative inline-flex items-center rounded-lg border border-gray-300 dark:border-gray-600 px-4 py-2 text-sm font-medium',
                        currentPage === 1
                            ? 'bg-gray-100 dark:bg-gray-800 text-gray-400 cursor-not-allowed'
                            : 'bg-white dark:bg-gray-900 text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-800'
                    )}
                >
                    Previous
                </button>
                <button
                    onClick={() => onPageChange(currentPage + 1)}
                    disabled={currentPage === totalPages}
                    className={cn(
                        'relative ml-3 inline-flex items-center rounded-lg border border-gray-300 dark:border-gray-600 px-4 py-2 text-sm font-medium',
                        currentPage === totalPages
                            ? 'bg-gray-100 dark:bg-gray-800 text-gray-400 cursor-not-allowed'
                            : 'bg-white dark:bg-gray-900 text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-800'
                    )}
                >
                    Next
                </button>
            </div>

            {/* Desktop View */}
            <div className="hidden sm:flex sm:flex-1 sm:items-center sm:justify-between">
                <div className="flex items-center gap-4">
                    <p className="text-sm text-gray-700 dark:text-gray-400">
                        Showing <span className="font-medium">{startIndex}</span> to{' '}
                        <span className="font-medium">{endIndex}</span> of{' '}
                        <span className="font-medium">{totalItems}</span> results
                    </p>

                    {onPageSizeChange && (
                        <div className="flex items-center gap-2">
                            <span className="text-sm text-gray-600 dark:text-gray-400">Per page:</span>
                            <select
                                value={itemsPerPage}
                                onChange={(e) => onPageSizeChange(parseInt(e.target.value))}
                                className="px-2 py-1 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            >
                                {pageSizeOptions.map((size) => (
                                    <option key={size} value={size}>
                                        {size}
                                    </option>
                                ))}
                            </select>
                        </div>
                    )}
                </div>

                <div>
                    <nav className="isolate inline-flex -space-x-px rounded-lg shadow-sm" aria-label="Pagination">
                        <button
                            onClick={() => onPageChange(currentPage - 1)}
                            disabled={currentPage === 1}
                            className={cn(
                                'relative inline-flex items-center rounded-l-lg px-2 py-2 text-gray-400 ring-1 ring-inset ring-gray-300 dark:ring-gray-600 hover:bg-gray-50 dark:hover:bg-gray-800 focus:z-20 focus:outline-offset-0',
                                currentPage === 1 && 'cursor-not-allowed opacity-50'
                            )}
                        >
                            <span className="sr-only">Previous</span>
                            <ChevronLeft className="h-4 w-4" aria-hidden="true" />
                        </button>

                        {/* First Page Link if needed */}
                        {startPage > 1 && (
                            <>
                                <button
                                    onClick={() => onPageChange(1)}
                                    className="relative inline-flex items-center px-4 py-2 text-sm font-semibold text-gray-900 dark:text-gray-200 ring-1 ring-inset ring-gray-300 dark:ring-gray-600 hover:bg-gray-50 dark:hover:bg-gray-800 focus:z-20 focus:outline-offset-0"
                                >
                                    1
                                </button>
                                {startPage > 2 && (
                                    <span className="relative inline-flex items-center px-4 py-2 text-sm font-semibold text-gray-700 dark:text-gray-400 ring-1 ring-inset ring-gray-300 dark:ring-gray-600 focus:outline-offset-0"
                                    >
                                        ...
                                    </span>
                                )}
                            </>
                        )}

                        {pages.map((page) => (
                            <button
                                key={page}
                                onClick={() => onPageChange(page)}
                                aria-current={page === currentPage ? 'page' : undefined}
                                className={cn(
                                    'relative inline-flex items-center px-4 py-2 text-sm font-semibold focus:z-20 focus:outline-offset-0',
                                    page === currentPage
                                        ? 'z-10 bg-blue-600 text-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600'
                                        : 'text-gray-900 dark:text-gray-200 ring-1 ring-inset ring-gray-300 dark:ring-gray-600 hover:bg-gray-50 dark:hover:bg-gray-800'
                                )}
                            >
                                {page}
                            </button>
                        ))}

                        {/* Last Page Link if needed */}
                        {endPage < totalPages && (
                            <>
                                {endPage < totalPages - 1 && (
                                    <span className="relative inline-flex items-center px-4 py-2 text-sm font-semibold text-gray-700 dark:text-gray-400 ring-1 ring-inset ring-gray-300 dark:ring-gray-600 focus:outline-offset-0"
                                    >
                                        ...
                                    </span>
                                )}
                                <button
                                    onClick={() => onPageChange(totalPages)}
                                    className="relative inline-flex items-center px-4 py-2 text-sm font-semibold text-gray-900 dark:text-gray-200 ring-1 ring-inset ring-gray-300 dark:ring-gray-600 hover:bg-gray-50 dark:hover:bg-gray-800 focus:z-20 focus:outline-offset-0"
                                >
                                    {totalPages}
                                </button>
                            </>
                        )}

                        <button
                            onClick={() => onPageChange(currentPage + 1)}
                            disabled={currentPage === totalPages}
                            className={cn(
                                'relative inline-flex items-center rounded-r-lg px-2 py-2 text-gray-400 ring-1 ring-inset ring-gray-300 dark:ring-gray-600 hover:bg-gray-50 dark:hover:bg-gray-800 focus:z-20 focus:outline-offset-0',
                                currentPage === totalPages && 'cursor-not-allowed opacity-50'
                            )}
                        >
                            <span className="sr-only">Next</span>
                            <ChevronRight className="h-4 w-4" aria-hidden="true" />
                        </button>
                    </nav>
                </div>
            </div>
        </div>
    );
}
