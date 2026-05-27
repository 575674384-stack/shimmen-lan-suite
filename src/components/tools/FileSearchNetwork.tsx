import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Search, User, Download, RefreshCw, FolderOpen, Settings2, X } from 'lucide-react';

interface FileIndexEntry {
  id: number;
  peer_id: string;
  peer_name: string;
  file_name: string;
  file_path: string;
  file_size: number;
  modified_at: number;
  is_local: boolean;
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
  return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB';
}

export default function FileSearchNetwork() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<FileIndexEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [indexing, setIndexing] = useState(false);
  const [, setIndexedCount] = useState(0);
  const [showSettings, setShowSettings] = useState(false);
  const [scanPaths, setScanPaths] = useState<string[]>([]);
  const [message, setMessage] = useState('');

  const loadDefaultPaths = async () => {
    try {
      const paths = await invoke<string[]>('get_indexed_directories');
      setScanPaths(paths);
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    loadDefaultPaths();
    // Listen for file search responses from remote peers
    const unlisten = listen('file-search-response', (event: any) => {
      if (event.payload?.results?.length > 0) {
        doSearch(query);
      }
    });
    return () => { unlisten.then((f) => f()); };
  }, []);

  const doSearch = useCallback(async (q: string) => {
    if (!q.trim()) {
      setResults([]);
      return;
    }
    setLoading(true);
    try {
      const data = await invoke<FileIndexEntry[]>('search_files_network', { query: q });
      setResults(data);
    } catch (e) {
      console.error(e);
    }
    setLoading(false);
  }, []);

  const handleSearch = (value: string) => {
    setQuery(value);
    if (value.trim()) {
      const timeout = setTimeout(() => doSearch(value), 300);
      return () => clearTimeout(timeout);
    } else {
      setResults([]);
    }
  };

  const handleRebuildIndex = async () => {
    setIndexing(true);
    setMessage('');
    try {
      const config = await invoke<{ device_id: string; username: string }>('get_config');
      const count = await invoke<number>('rebuild_file_index', {
        paths: scanPaths,
        peer_id: config.device_id,
        peer_name: config.username,
      });
      setIndexedCount(count);
      setMessage(`索引完成，共 ${count} 个文件`);
    } catch (e) {
      setMessage('索引失败: ' + String(e));
    }
    setIndexing(false);
  };

  const handleRequestFile = async (entry: FileIndexEntry) => {
    if (entry.is_local) {
      // Open local file location
      try {
        await invoke('open_file_location', { path: entry.file_path });
      } catch (e) {
        console.error(e);
      }
      return;
    }
    try {
      await invoke('request_file_from_peer', {
        peer_id: entry.peer_id,
        file_path: entry.file_path,
      });
      setMessage(`已向 ${entry.peer_name} 请求传输: ${entry.file_name}`);
    } catch (e) {
      setMessage('请求失败: ' + String(e));
    }
  };

  const addPath = async () => {
    try {
      const path = await invoke<string | null>('select_folder');
      if (path && !scanPaths.includes(path)) {
        setScanPaths([...scanPaths, path]);
      }
    } catch (e) {
      console.error(e);
    }
  };

  const removePath = (path: string) => {
    setScanPaths(scanPaths.filter((p) => p !== path));
  };

  const localCount = results.filter((r) => r.is_local).length;
  const remoteCount = results.filter((r) => !r.is_local).length;

  return (
    <div className="max-w-4xl mx-auto space-y-4">
      {/* 搜索栏 */}
      <div className="flex gap-3">
        <div className="flex-1 relative">
          <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-text-secondary" />
          <input
            type="text"
            value={query}
            onChange={(e) => handleSearch(e.target.value)}
            placeholder="搜索文件名（支持模糊匹配）..."
            className="w-full pl-10 pr-4 py-2.5 bg-background border border-border rounded-xl text-base focus:outline-none focus:ring-2 focus:ring-primary/20"
          />
        </div>
        <button
          onClick={() => setShowSettings(!showSettings)}
          className="flex items-center gap-1.5 px-3 py-2.5 bg-surface border border-border rounded-xl text-sm hover:border-primary transition-colors"
        >
          <Settings2 size={16} />
          设置
        </button>
        <button
          onClick={handleRebuildIndex}
          disabled={indexing}
          className="flex items-center gap-1.5 px-3 py-2.5 bg-primary text-white rounded-xl text-sm hover:bg-primary-dark transition-colors disabled:opacity-50"
        >
          <RefreshCw size={16} className={indexing ? 'animate-spin' : ''} />
          {indexing ? '索引中...' : '重建索引'}
        </button>
      </div>

      {/* 统计 */}
      {query && (
        <div className="flex gap-4 text-sm text-text-secondary">
          <span>本地: <span className="text-text-primary font-medium">{localCount}</span></span>
          <span>其他电脑: <span className="text-text-primary font-medium">{remoteCount}</span></span>
        </div>
      )}

      {/* 设置面板 */}
      {showSettings && (
        <div className="bg-surface rounded-xl p-4 border border-border space-y-3">
          <div className="flex items-center justify-between">
            <h4 className="font-medium text-text-primary">扫描目录</h4>
            <button onClick={() => setShowSettings(false)} className="text-text-secondary hover:text-text-primary">
              <X size={16} />
            </button>
          </div>
          <div className="space-y-2">
            {scanPaths.map((path) => (
              <div key={path} className="flex items-center justify-between bg-background rounded-lg px-3 py-2 text-sm">
                <span className="text-text-primary truncate">{path}</span>
                <button onClick={() => removePath(path)} className="text-text-secondary hover:text-red-500">
                  <X size={14} />
                </button>
              </div>
            ))}
          </div>
          <button
            onClick={addPath}
            className="w-full py-2 border border-dashed border-border rounded-lg text-sm text-text-secondary hover:text-primary hover:border-primary transition-colors"
          >
            + 添加目录
          </button>
        </div>
      )}

      {/* 结果列表 */}
      {loading && <div className="text-center py-8 text-text-secondary text-sm">搜索中...</div>}

      {!loading && query && results.length === 0 && (
        <div className="text-center py-8 text-text-secondary text-sm">未找到匹配的文件</div>
      )}

      {results.length > 0 && (
        <div className="bg-surface rounded-xl border border-border overflow-hidden">
          <div className="grid grid-cols-[1fr_100px_120px_100px_80px] gap-2 px-4 py-2 bg-background text-xs text-text-secondary font-medium">
            <span>文件名</span>
            <span>大小</span>
            <span>所在电脑</span>
            <span>位置</span>
            <span></span>
          </div>
          <div className="max-h-[450px] overflow-auto">
            {results.map((entry) => (
              <div
                key={entry.id}
                className="grid grid-cols-[1fr_100px_120px_100px_80px] gap-2 px-4 py-2.5 border-t border-border hover:bg-background transition-colors items-center"
              >
                <div className="min-w-0">
                  <div className="text-sm text-text-primary truncate" title={entry.file_name}>{entry.file_name}</div>
                  <div className="text-xs text-text-secondary truncate" title={entry.file_path}>{entry.file_path}</div>
                </div>
                <span className="text-xs text-text-secondary">{formatSize(entry.file_size)}</span>
                <div className="flex items-center gap-1.5">
                  <User size={12} className="text-text-secondary" />
                  <span className="text-xs text-text-secondary truncate">
                    {entry.is_local ? '本机' : entry.peer_name || entry.peer_id.slice(0, 6)}
                  </span>
                </div>
                <span className="text-xs text-text-secondary">{entry.is_local ? '本地' : '远程'}</span>
                <button
                  onClick={() => handleRequestFile(entry)}
                  className="flex items-center justify-center gap-1 px-2 py-1 text-xs bg-primary text-white rounded-lg hover:bg-primary-dark transition-colors"
                  title={entry.is_local ? '打开所在位置' : '请求传输'}
                >
                  {entry.is_local ? <FolderOpen size={12} /> : <Download size={12} />}
                  {entry.is_local ? '打开' : '获取'}
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {message && (
        <div className="text-sm text-center py-2 rounded-lg bg-green-50 text-green-600">
          {message}
        </div>
      )}
    </div>
  );
}
