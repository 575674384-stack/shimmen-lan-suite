import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { X, Palette, User, Image } from 'lucide-react';
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

export default function SettingsModal({ isOpen, onClose }: SettingsModalProps) {
  const [username, setUsername] = useState('');
  const [avatarPreset, setAvatarPreset] = useState('');
  const [avatarBase64, setAvatarBase64] = useState('');
  const [currentTheme, setCurrentTheme] = useState(() => {
    return localStorage.getItem('theme') || 'default';
  });

  useEffect(() => {
    const saved = localStorage.getItem('theme');
    if (saved) {
      document.documentElement.setAttribute('data-theme', saved === 'default' ? '' : saved);
    }
  }, []);

  const loadSettings = async () => {
    try {
      const config = await invoke<{ username: string; avatar_preset: string }>('get_config');
      setUsername(config.username);
      setAvatarPreset(config.avatar_preset || '');
    } catch {
      setUsername('');
      setAvatarPreset('');
    }
    try {
      const avatar = await invoke<string>('get_avatar');
      setAvatarBase64(avatar);
    } catch {
      setAvatarBase64('');
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

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/30 flex items-center justify-center z-50">
      <div className="bg-surface rounded-2xl p-6 w-[420px] shadow-xl max-h-[90vh] overflow-auto">
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
      </div>
    </div>
  );
}
