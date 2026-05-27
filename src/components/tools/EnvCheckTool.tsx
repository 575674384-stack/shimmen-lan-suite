import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Check, X, Download, RefreshCw, ExternalLink } from 'lucide-react';

interface SoftwareInfo {
  id: string;
  name: string;
  installed: boolean;
  version: string;
  install_path: string;
  download_url: string;
  installer_args: string;
}

const SOFTWARE_ICONS: Record<string, string> = {
  dotnet: '📦',
  vcredist: '🔧',
  java: '☕',
  python: '🐍',
  nodejs: '⬢',
  git: '🌿',
  chrome: '🌐',
  '7zip': '📂',
  winrar: '📦',
  office: '📝',
};

export default function EnvCheckTool() {
  const [software, setSoftware] = useState<SoftwareInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [installing, setInstalling] = useState<string | null>(null);
  const [message, setMessage] = useState('');

  const loadSoftware = async () => {
    setLoading(true);
    try {
      const data = await invoke<SoftwareInfo[]>('get_installed_software');
      setSoftware(data);
    } catch (e) {
      console.error(e);
    }
    setLoading(false);
  };

  useEffect(() => {
    loadSoftware();
  }, []);

  const handleInstall = async (sw: SoftwareInfo) => {
    if (!sw.download_url) {
      setMessage(`${sw.name} 请手动下载安装`);
      return;
    }
    setInstalling(sw.id);
    setMessage('');
    try {
      const result = await invoke<string>('install_software', {
        download_url: sw.download_url,
        installer_args: sw.installer_args,
      });
      setMessage(result);
      loadSoftware();
    } catch (e) {
      setMessage('安装失败: ' + String(e));
    }
    setInstalling(null);
  };

  const openUrl = (url: string) => {
    window.open(url, '_blank');
  };

  const installedCount = software.filter((s) => s.installed).length;

  return (
    <div className="max-w-3xl mx-auto space-y-5">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="font-semibold text-lg text-text-primary">常用依赖检测</h3>
          <p className="text-sm text-text-secondary">
            已安装 {installedCount} / {software.length} 项
          </p>
        </div>
        <button
          onClick={loadSoftware}
          disabled={loading}
          className="flex items-center gap-1.5 px-3 py-1.5 text-sm bg-primary text-white rounded-lg hover:bg-primary-dark transition-colors disabled:opacity-50"
        >
          <RefreshCw size={14} className={loading ? 'animate-spin' : ''} />
          刷新
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        {software.map((sw) => (
          <div
            key={sw.id}
            className={`bg-surface rounded-xl p-4 border transition-colors ${
              sw.installed ? 'border-green-200' : 'border-border'
            }`}
          >
            <div className="flex items-start gap-3">
              <div className="w-10 h-10 rounded-lg bg-background flex items-center justify-center text-xl shrink-0">
                {SOFTWARE_ICONS[sw.id] || '📦'}
              </div>
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="font-medium text-text-primary">{sw.name}</span>
                  {sw.installed ? (
                    <span className="flex items-center gap-0.5 text-xs text-green-600">
                      <Check size={12} /> 已安装
                    </span>
                  ) : (
                    <span className="flex items-center gap-0.5 text-xs text-red-500">
                      <X size={12} /> 未安装
                    </span>
                  )}
                </div>
                {sw.version && (
                  <div className="text-xs text-text-secondary mt-0.5">{sw.version}</div>
                )}
              </div>
            </div>

            {!sw.installed && (
              <div className="flex gap-2 mt-3">
                {sw.download_url && (
                  <button
                    onClick={() => handleInstall(sw)}
                    disabled={installing === sw.id}
                    className="flex-1 flex items-center justify-center gap-1.5 px-3 py-1.5 bg-primary text-white text-sm rounded-lg hover:bg-primary-dark transition-colors disabled:opacity-50"
                  >
                    <Download size={14} />
                    {installing === sw.id ? '安装中...' : '一键安装'}
                  </button>
                )}
                <button
                  onClick={() => openUrl(sw.download_url || '#')}
                  className="px-3 py-1.5 bg-surface border border-border text-text-secondary rounded-lg hover:bg-background transition-colors"
                  title="打开官网"
                >
                  <ExternalLink size={14} />
                </button>
              </div>
            )}
          </div>
        ))}
      </div>

      {message && (
        <div className={`text-sm text-center py-2 rounded-lg ${message.includes('失败') ? 'text-red-500 bg-red-50' : 'text-green-600 bg-green-50'}`}>
          {message}
        </div>
      )}
    </div>
  );
}
