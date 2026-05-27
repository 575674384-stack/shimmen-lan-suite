import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Upload, Check } from 'lucide-react';

interface AvatarSettingsProps {
  currentPreset?: string;
  currentAvatarBase64?: string;
  onSaved?: () => void;
}

const presets = [
  { id: 'cat', emoji: '🐱', gradient: 'from-orange-400 to-pink-500' },
  { id: 'dog', emoji: '🐶', gradient: 'from-blue-400 to-purple-500' },
  { id: 'fox', emoji: '🦊', gradient: 'from-red-400 to-yellow-500' },
  { id: 'panda', emoji: '🐼', gradient: 'from-gray-700 to-gray-300' },
  { id: 'rabbit', emoji: '🐰', gradient: 'from-pink-300 to-white' },
  { id: 'tiger', emoji: '🐯', gradient: 'from-orange-500 to-yellow-400' },
  { id: 'frog', emoji: '🐸', gradient: 'from-green-400 to-emerald-600' },
  { id: 'octopus', emoji: '🐙', gradient: 'from-purple-500 to-red-500' },
];

export default function AvatarSettings({ currentPreset = '', currentAvatarBase64 = '', onSaved }: AvatarSettingsProps) {
  const [selectedPreset, setSelectedPreset] = useState(currentPreset);
  const [customBase64, setCustomBase64] = useState('');
  const [previewBase64, setPreviewBase64] = useState(currentAvatarBase64);
  const [saving, setSaving] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    setSelectedPreset(currentPreset);
    setPreviewBase64(currentAvatarBase64);
    setCustomBase64('');
  }, [currentPreset, currentAvatarBase64]);

  const handlePresetClick = (id: string) => {
    setSelectedPreset(id);
    setCustomBase64('');
    setPreviewBase64('');
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (event) => {
      const result = event.target?.result as string;
      setCustomBase64(result);
      setPreviewBase64(result);
      setSelectedPreset('');
    };
    reader.readAsDataURL(file);
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      if (customBase64) {
        await invoke('set_avatar', { path: customBase64 });
      }
      await invoke('set_avatar_preset', { preset: selectedPreset });
      onSaved?.();
    } catch (e) {
      alert('保存头像失败: ' + (e instanceof Error ? e.message : String(e)));
    } finally {
      setSaving(false);
    }
  };

  const renderPreview = () => {
    if (previewBase64) {
      return (
        <img
          src={previewBase64}
          alt="头像预览"
          className="w-full h-full object-cover rounded-full"
        />
      );
    }
    if (selectedPreset) {
      const preset = presets.find(p => p.id === selectedPreset);
      if (preset) {
        return (
          <div className={`w-full h-full bg-gradient-to-br ${preset.gradient} rounded-full flex items-center justify-center text-2xl`}>
            {preset.emoji}
          </div>
        );
      }
    }
    return (
      <div className="w-full h-full bg-gradient-to-br from-sky-400 to-blue-500 rounded-full flex items-center justify-center text-white text-sm">
        ?
      </div>
    );
  };

  return (
    <div className="space-y-4">
      {/* 预览 */}
      <div className="flex justify-center">
        <div className="w-20 h-20 rounded-full border-2 border-border overflow-hidden shadow-sm">
          {renderPreview()}
        </div>
      </div>

      {/* 预设选择 */}
      <div>
        <label className="text-sm font-medium text-text-primary mb-2 block">选择预设头像</label>
        <div className="grid grid-cols-4 gap-2">
          {presets.map((preset) => (
            <button
              key={preset.id}
              onClick={() => handlePresetClick(preset.id)}
              className={`relative aspect-square rounded-xl flex items-center justify-center text-2xl transition-all ${
                selectedPreset === preset.id
                  ? 'ring-2 ring-primary scale-105'
                  : 'hover:scale-105'
              } bg-gradient-to-br ${preset.gradient}`}
              title={preset.emoji}
            >
              <span className="drop-shadow-sm">{preset.emoji}</span>
              {selectedPreset === preset.id && (
                <div className="absolute bottom-0.5 right-0.5 w-4 h-4 bg-primary rounded-full flex items-center justify-center">
                  <Check size={10} className="text-white" />
                </div>
              )}
            </button>
          ))}
        </div>
      </div>

      {/* 自定义上传 */}
      <div>
        <label className="text-sm font-medium text-text-primary mb-2 block">自定义头像</label>
        <button
          onClick={() => fileInputRef.current?.click()}
          className="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-background border border-border border-dashed rounded-xl text-sm text-text-secondary hover:bg-surface hover:text-text-primary transition-colors"
        >
          <Upload size={16} />
          {customBase64 ? '更换图片' : '上传图片'}
        </button>
        <input
          ref={fileInputRef}
          type="file"
          accept="image/*"
          onChange={handleFileChange}
          className="hidden"
        />
        {customBase64 && (
          <p className="text-xs text-text-secondary mt-1 text-center">已选择图片，点击保存生效</p>
        )}
      </div>

      {/* 保存按钮 */}
      <button
        onClick={handleSave}
        disabled={saving}
        className="w-full px-4 py-2 bg-primary text-white text-sm rounded-xl hover:bg-primary-dark transition-colors disabled:opacity-50"
      >
        {saving ? '保存中...' : '保存头像'}
      </button>
    </div>
  );
}
