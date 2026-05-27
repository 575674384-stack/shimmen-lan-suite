import { useState, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Search, FolderOpen, FileText, ExternalLink } from 'lucide-react';

interface FileSearchResult {
  name: string;
  path: string;
  size: number;
  modified: string;
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
  return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB';
}

export default function FileSearchTool() {
  const [path, setPath] = useState('C:\\');
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<FileSearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [searched, setSearched] = useState(false);
  const timeoutRef = useRef<ReturnType<typeof setTimeout>>();

  const doSearch = useCallback(async (searchQuery: string, searchPath: string) => {
    if (!searchQuery.trim()) {
      setResults([]);
      setSearched(false);
      return;
    }
    setLoading(true);
    setSearched(true);
    try {
      const data = await invoke<FileSearchResult[]>('search_files', {
        path: searchPath,
        query: searchQuery,
        limit: 50,
      });
      setResults(data);
    } catch (e) {
      console.error(e);
      setResults([]);
    }
    setLoading(false);
  }, []);

  const handleQueryChange = (value: string) => {
    setQuery(value);
    if (timeoutRef.current) clearTimeout(timeoutRef.current);
    timeoutRef.current = setTimeout(() => {
      doSearch(value, path);
    }, 500);
  };

  const selectPath = async () => {
    try {
      const selected = await invoke<string | null>('select_folder');
      if (selected) {
        setPath(selected);
        if (query) doSearch(query, selected);
      }
    } catch (e) {
      console.error(e);
    }
  };

  const openLocation = async (filePath: string) => {
    try {
      await invoke('open_file_location', { path: filePath });
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div className="max-w-3xl mx-auto space-y-4">
      <div className="flex gap-3">
        <button
          onClick={selectPath}
          className="flex items-center gap-2 px-4 py-2.5 bg-surface border border-border rounded-xl text-sm hover:border-primary transition-colors shrink-0"
        >
          <FolderOpen size={16} />
          <span className="max-w-[200px] truncate">{path}</span>
        </button>
        <div className="flex-1 relative">
          <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-text-secondary" />
          <input
            type="text"
            value={query}
            onChange={(e) => handleQueryChange(e.target.value)}
            placeholder="输入文件名关键词搜索..."
            className="w-full pl-10 pr-4 py-2.5 bg-background border border-border rounded-xl text-base focus:outline-none focus:ring-2 focus:ring-primary/20"
          />
        </div>
      </div>

      {loading && (
        <div className="text-center py-8 text-text-secondary text-sm">搜索中...</div>
      )}

      {!loading && searched && results.length === 0 && (
        <div className="text-center py-8 text-text-secondary text-sm">未找到匹配的文件</div>
      )}

      {results.length > 0 && (
        <div className="bg-surface rounded-xl border border-border overflow-hidden">
          <div className="grid grid-cols-[1fr_100px_140px_48px] gap-2 px-4 py-2 bg-background text-xs text-text-secondary font-medium">
            <span>文件名</span>
            <span>大小</span>
            <span>修改时间</span>
            <span></span>
          </div>
          <div className="max-h-[400px] overflow-auto">
            {results.map((file, i) => (
              <div
                key={i}
                className="grid grid-cols-[1fr_100px_140px_48px] gap-2 px-4 py-2.5 border-t border-border hover:bg-background transition-colors items-center"
              >
                <div className="min-w-0 flex items-center gap-2">
                  <FileText size={14} className="text-text-secondary shrink-0" />
                  <span className="text-sm text-text-primary truncate" title={file.name}>{file.name}</span>
                </div>
                <span className="text-xs text-text-secondary">{formatSize(file.size)}</span>
                <span className="text-xs text-text-secondary">{file.modified}</span>
                <button
                  onClick={() => openLocation(file.path)}
                  className="p-1.5 rounded-lg hover:bg-background text-text-secondary hover:text-primary transition-colors"
                  title="打开所在位置"
                >
                  <ExternalLink size={14} />
                </button>
              </div>
            ))}
          </div>
          <div className="px-4 py-2 bg-background border-t border-border text-xs text-text-secondary text-right">
            共找到 {results.length} 个文件
          </div>
        </div>
      )}
    </div>
  );
}
