import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export function useAvatar() {
  const [avatarBase64, setAvatarBase64] = useState('');
  const [avatarPreset, setAvatarPreset] = useState('');
  const [username, setUsername] = useState('');
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const config = await invoke<{ username: string; avatar_preset: string }>('get_config');
      setUsername(config.username);
      setAvatarPreset(config.avatar_preset || '');
      const avatar = await invoke<string>('get_avatar');
      setAvatarBase64(avatar);
    } catch (e) {
      console.error('加载头像失败:', e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { avatarBase64, avatarPreset, username, loading, refresh };
}
