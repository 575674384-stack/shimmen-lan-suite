import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Terminal, Check, Download, RefreshCw, AlertCircle } from 'lucide-react';

export default function PwshTool() {
  const [ps7Installed, setPs7Installed] = useState<boolean | null>(null);
  const [utf8Enabled, setUtf8Enabled] = useState<boolean | null>(null);
  const [installing, setInstalling] = useState(false);
  const [settingUtf8, setSettingUtf8] = useState(false);
  const [message, setMessage] = useState('');

  const checkStatus = async () => {
    try {
      const ps7 = await invoke<boolean>('check_powershell7');
      const utf8 = await invoke<boolean>('check_utf8');
      setPs7Installed(ps7);
      setUtf8Enabled(utf8);
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    checkStatus();
  }, []);

  const handleInstallPs7 = async () => {
    setInstalling(true);
    setMessage('');
    try {
      const result = await invoke<string>('install_powershell7');
      setMessage(result);
      checkStatus();
    } catch (e) {
      setMessage('安装失败: ' + String(e));
    }
    setInstalling(false);
  };

  const handleSetUtf8 = async (enabled: boolean) => {
    setSettingUtf8(true);
    setMessage('');
    try {
      await invoke('set_utf8', { enabled });
      setMessage(`UTF-8 已${enabled ? '开启' : '关闭'}，重启后生效`);
      checkStatus();
    } catch (e) {
      setMessage('设置失败: ' + String(e));
    }
    setSettingUtf8(false);
  };

  return (
    <div className="max-w-2xl mx-auto space-y-6">
      {/* PowerShell 7 */}
      <div className="bg-surface rounded-2xl p-5 border border-border space-y-4">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-blue-100 flex items-center justify-center">
            <Terminal size={22} className="text-blue-600" />
          </div>
          <div>
            <h4 className="font-semibold text-base text-text-primary">PowerShell 7</h4>
            <p className="text-sm text-text-secondary">跨平台现代命令行工具</p>
          </div>
          <div className="ml-auto">
            {ps7Installed === null ? (
              <span className="text-sm text-text-secondary">检测中...</span>
            ) : ps7Installed ? (
              <span className="flex items-center gap-1 text-sm text-green-600">
                <Check size={14} /> 已安装
              </span>
            ) : (
              <span className="flex items-center gap-1 text-sm text-amber-600">
                <AlertCircle size={14} /> 未安装
              </span>
            )}
          </div>
        </div>

        {!ps7Installed && (
          <button
            onClick={handleInstallPs7}
            disabled={installing}
            className="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-blue-500 text-white rounded-xl hover:bg-blue-600 transition-colors disabled:opacity-50"
          >
            <Download size={16} />
            {installing ? '下载安装中...' : '安装 PowerShell 7'}
          </button>
        )}
      </div>

      {/* UTF-8 设置 */}
      <div className="bg-surface rounded-2xl p-5 border border-border space-y-4">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-emerald-100 flex items-center justify-center">
            <RefreshCw size={22} className="text-emerald-600" />
          </div>
          <div>
            <h4 className="font-semibold text-base text-text-primary">系统 UTF-8 编码</h4>
            <p className="text-sm text-text-secondary">解决中文乱码问题，修改 CodePage 为 UTF-8</p>
          </div>
          <div className="ml-auto">
            {utf8Enabled === null ? (
              <span className="text-sm text-text-secondary">检测中...</span>
            ) : utf8Enabled ? (
              <span className="flex items-center gap-1 text-sm text-green-600">
                <Check size={14} /> 已开启
              </span>
            ) : (
              <span className="flex items-center gap-1 text-sm text-amber-600">
                <AlertCircle size={14} /> 未开启
              </span>
            )}
          </div>
        </div>

        <div className="flex gap-3">
          <button
            onClick={() => handleSetUtf8(true)}
            disabled={settingUtf8 || utf8Enabled === true}
            className="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 bg-emerald-500 text-white rounded-xl hover:bg-emerald-600 transition-colors disabled:opacity-50"
          >
            开启 UTF-8
          </button>
          <button
            onClick={() => handleSetUtf8(false)}
            disabled={settingUtf8 || utf8Enabled === false}
            className="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 bg-surface border border-border text-text-secondary rounded-xl hover:bg-background transition-colors disabled:opacity-50"
          >
            恢复默认 (GBK)
          </button>
        </div>
      </div>

      {message && (
        <div className={`text-sm text-center py-2 rounded-lg ${message.includes('失败') ? 'text-red-500 bg-red-50' : 'text-green-600 bg-green-50'}`}>
          {message}
        </div>
      )}
    </div>
  );
}
