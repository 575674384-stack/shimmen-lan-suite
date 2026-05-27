import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { X, Palette, User, Image, FolderOpen, Clock, Power, Monitor, RefreshCw } from 'lucide-react';
import AvatarSettings from '../settings/AvatarSettings';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

const themes = [
  { id: 'default', name: '青色', color: '#0ea5e9' },
  { id: 'blue', name: '蓝色', color: '#3b82f6' },
  { id: 'purple', name: '紫色', color: '#8b5cf6' },
  { id: 'green', name: '绿色', color: '#10b981' },
  { id: 'orange', name: '橙色', color: '#f59e0b' },
];

const syncIntervals = [
  { value: 0, label: '实时' },
  { value: 300, label: '5 分钟' },
  { value: 1800, label: '30 分钟' },
];

const screenFpsOptions = [
  { value: 5, label: '5 FPS' },
  { value: 10, label: '10 FPS' },
  { value: 15, label: '15 FPS' },
  { value: 30, label: '30 FPS' },
];

const screenResOptions = [
  { value: 450, label: '450p' },
  { value: 540, label: '540p' },
  { value: 720, label: '720p' },
];

export default function SettingsModal({ isOpen, onClose }: SettingsModalProps) {
  const [username, setUsername] = useState('');
  const [avatarPreset, setAvatarPreset] = useState('');
  const [avatarBase64, setAvatarBase64] = useState('');
  const [currentTheme, setCurrentTheme] = useState(() => {
    return localStorage.getItem('theme') || 'default';
  });
  const [downloadDir, setDownloadDir] = useState('');
  const [syncInterval, setSyncInterval] = useState(0);
  const [autostart, setAutostart] = useState(false);
  const [screenFps, setScreenFps] = useState(10);
  const [screenRes, setScreenRes] = useState(720);
  const [autoUpdate, setAutoUpdate] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    const saved = localStorage.getItem('theme');
    if (saved) {
      document.documentElement.setAttribute('data-theme', saved === 'default' ? '' : saved);
    }
  }, []);

  const loadSettings = async () => {
    try {
      const config = await invoke<{
        username: string;
        avatar_preset: string;
        download_dir: string;
        sync_interval_secs: number;
        autostart: boolean;
        screen_fps: number;
        screen_resolution: number;
        auto_update: boolean;
      }>('get_config');
      setUsername(config.username);
      setAvatarPreset(config.avatar_preset || '');
      setDownloadDir(config.download_dir || '');
      setSyncInterval(config.sync_interval_secs ?? 0);
      setAutostart(config.autostart ?? false);
      setScreenFps(config.screen_fps ?? 10);
      setScreenRes(config.screen_resolution ?? 720);
      setAutoUpdate(config.auto_update ?? true);
    } catch {
      setUsername('');
      setAvatarPreset('');
      setDownloadDir('');
      setSyncInterval(0);
      setAutostart(false);
    }
    try {
      const avatar = await invoke<string>('get_avatar');
      setAvatarBase64(avatar);
    } catch {
      setAvatarBase64('');
    }
    try {
      const status = await invoke<boolean>('get_autostart_status');
      setAutostart(status);
    } catch {
      // ignore
    }
  };

  useEffect(() => {
    if (isOpen) {
      loadSettings();
    }
  }, [isOpen]);

  const handleSaveUsername = async () => {
    try {
      await invoke('set_username', { username });
    } catch (_e) {
      // ignore
    }
    onClose();
    window.location.reload();
  };

  const handleThemeChange = (themeId: string) => {
    setCurrentTheme(themeId);
    document.documentElement.setAttribute('data-theme', themeId === 'default' ? '' : themeId);
    localStorage.setItem('theme', themeId);
  };

  const handleSelectDownloadDir = async () => {
    try {
      const path = await invoke<string | null>('select_folder');
      if (path) {
        setDownloadDir(path);
      }
    } catch (_e) {
      // ignore
    }
  };

  const handleSaveSettings = async () => {
    setSaving(true);
    try {
      await invoke('set_download_dir', { downloadDir: downloadDir });
      await invoke('set_sync_interval', { syncIntervalSecs: syncInterval });
      await invoke('set_autostart', { enabled: autostart });
      await invoke('set_screen_fps', { fps: screenFps });
      await invoke('set_screen_resolution', { resolution: screenRes });
      await invoke('set_auto_update', { enabled: autoUpdate });
      onClose();
      window.location.reload();
    } catch (e) {
      alert('保存设置失败: ' + (e instanceof Error ? e.message : String(e)));
    } finally {
      setSaving(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/30 flex items-center justify-center z-50">
      <div className="bg-surface rounded-2xl p-6 w-[460px] shadow-xl max-h-[90vh] overflow-auto">
        <div className="flex items-center justify-between mb-6">
          <h3 className="font-bold text-lg text-text-primary">设置</h3>
          <button onClick={onClose} className="p-1 text-text-secondary hover:text-text-primary rounded-lg hover:bg-background transition-colors">
            <X size={18} />
          </button>
        </div>

        {/* 用户名设置 */}
        <div className="mb-6">
          <label className="flex items-center gap-2 text-sm font-medium text-text-primary mb-2">
            <User size={16} />
            用户名
          </label>
          <div className="flex gap-2">
            <input
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              className="flex-1 px-3 py-2 bg-background border border-border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary/20"
            />
            <button
              onClick={handleSaveUsername}
              className="px-4 py-2 bg-primary text-white text-sm rounded-xl hover:bg-primary-dark transition-colors"
            >
              保存
            </button>
          </div>
        </div>

        {/* 头像设置 */}
        <div className="mb-6">
          <label className="flex items-center gap-2 text-sm font-medium text-text-primary mb-2">
            <Image size={16} />
            头像
          </label>
          <AvatarSettings
            currentPreset={avatarPreset}
            currentAvatarBase64={avatarBase64}
            onSaved={() => {
              loadSettings();
              window.location.reload();
            }}
          />
        </div>

        {/* 主题色设置 */}
        <div className="mb-6">
          <label className="flex items-center gap-2 text-sm font-medium text-text-primary mb-3">
            <Palette size={16} />
            主题色
          </label>
          <div className="flex gap-2">
            {themes.map((theme) => (
              <button
                key={theme.id}
                onClick={() => handleThemeChange(theme.id)}
                className={`w-8 h-8 rounded-full transition-transform ${currentTheme === theme.id ? 'ring-2 ring-offset-2 ring-primary scale-110' : 'hover:scale-105'}`}
                style={{ backgroundColor: theme.color }}
                title={theme.name}
              />
            ))}
          </div>
        </div>

        {/* 文件接收路径 */}
        <div className="mb-6">
          <label className="flex items-center gap-2 text-sm font-medium text-text-primary mb-2">
            <FolderOpen size={16} />
            文件接收路径
          </label>
          <div className="flex gap-2">
            <input
              type="text"
              value={downloadDir}
              readOnly
              placeholder="默认: 应用数据目录/downloads"
              className="flex-1 px-3 py-2 bg-background border border-border rounded-xl text-sm text-text-secondary focus:outline-none"
            />
            <button
              onClick={handleSelectDownloadDir}
              className="px-3 py-2 bg-background border border-border text-text-primary text-sm rounded-xl hover:bg-surface transition-colors"
            >
              浏览
            </button>
          </div>
          <p className="text-xs text-text-secondary mt-1">点对点文件传输的默认保存位置</p>
        </div>

        {/* 共享文件夹同步间隔 */}
        <div className="mb-6">
          <label className="flex items-center gap-2 text-sm font-medium text-text-primary mb-2">
            <Clock size={16} />
            共享文件夹同步间隔
          </label>
          <div className="flex gap-2">
            {syncIntervals.map((item) => (
              <button
                key={item.value}
                onClick={() => setSyncInterval(item.value)}
                className={`flex-1 px-3 py-2 text-sm rounded-xl border transition-colors ${
                  syncInterval === item.value
                    ? 'bg-primary text-white border-primary'
                    : 'bg-background text-text-primary border-border hover:bg-surface'
                }`}
              >
                {item.label}
              </button>
            ))}
          </div>
        </div>

        {/* 开机自启 */}
        <div className="mb-6">
          <label className="flex items-center gap-2 text-sm font-medium text-text-primary mb-2">
            <Power size={16} />
            开机自启
          </label>
          <div className="flex items-center gap-3">
            <button
              onClick={() => setAutostart(!autostart)}
              className={`relative w-11 h-6 rounded-full transition-colors ${autostart ? 'bg-primary' : 'bg-border'}`}
            >
              <span
                className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform ${autostart ? 'translate-x-5' : ''}`}
              />
            </button>
            <span className="text-sm text-text-secondary">{autostart ? '已开启' : '已关闭'}</span>
          </div>
        </div>

        {/* 自动检查更新 */}
        <div className="mb-6">
          <label className="flex items-center gap-2 text-sm font-medium text-text-primary mb-2">
            <RefreshCw size={16} />
            自动检查更新
          </label>
          <div className="flex items-center gap-3">
            <button
              onClick={() => setAutoUpdate(!autoUpdate)}
              className={`relative w-11 h-6 rounded-full transition-colors ${autoUpdate ? 'bg-primary' : 'bg-border'}`}
            >
              <span
                className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform ${autoUpdate ? 'translate-x-5' : ''}`}
              />
            </button>
            <span className="text-sm text-text-secondary">{autoUpdate ? '已开启' : '已关闭'}</span>
          </div>
        </div>

        {/* 截屏分享质量 */}
        <div className="mb-6">
          <label className="flex items-center gap-2 text-sm font-medium text-text-primary mb-2">
            <Monitor size={16} />
            截屏分享质量
          </label>
          <div className="grid grid-cols-2 gap-2 mb-2">
            <div>
              <span className="text-xs text-text-secondary mb-1 block">帧率</span>
              <div className="flex gap-1">
                {screenFpsOptions.map((opt) => (
                  <button
                    key={opt.value}
                    onClick={() => setScreenFps(opt.value)}
                    className={`flex-1 px-2 py-1.5 text-xs rounded-lg border transition-colors ${
                      screenFps === opt.value
                        ? 'bg-primary text-white border-primary'
                        : 'bg-background text-text-primary border-border hover:bg-surface'
                    }`}
                  >
                    {opt.label}
                  </button>
                ))}
              </div>
            </div>
            <div>
              <span className="text-xs text-text-secondary mb-1 block">分辨率</span>
              <div className="flex gap-1">
                {screenResOptions.map((opt) => (
                  <button
                    key={opt.value}
                    onClick={() => setScreenRes(opt.value)}
                    className={`flex-1 px-2 py-1.5 text-xs rounded-lg border transition-colors ${
                      screenRes === opt.value
                        ? 'bg-primary text-white border-primary'
                        : 'bg-background text-text-primary border-border hover:bg-surface'
                    }`}
                  >
                    {opt.label}
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>

        {/* 保存所有设置 */}
        <button
          onClick={handleSaveSettings}
          disabled={saving}
          className="w-full px-4 py-2.5 bg-primary text-white text-sm font-medium rounded-xl hover:bg-primary-dark transition-colors disabled:opacity-50"
        >
          {saving ? '保存中...' : '保存所有设置'}
        </button>
      </div>
    </div>
  );
}
