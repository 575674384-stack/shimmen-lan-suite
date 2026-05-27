import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { SharedFolder } from '../types';

export function useFolders() {
  const [myFolders, setMyFolders] = useState<SharedFolder[]>([]);
  const [remoteFolders, setRemoteFolders] = useState<SharedFolder[]>([]);

  const loadFolders = async () => {
    try {
      const my = await invoke<SharedFolder[]>('get_my_shared_folders');
      const remote = await invoke<SharedFolder[]>('get_remote_shared_folders');
      setMyFolders(my);
      setRemoteFolders(remote);
    } catch (e) {
      console.error('加载共享文件夹失败:', e);
    }
  };

  const createFolder = async (name: string, localPath: string) => {
    try {
      await invoke('create_shared_folder', { name, local_path: localPath });
      await loadFolders();
    } catch (e) {
      console.error('创建共享文件夹失败:', e);
      alert('创建共享文件夹失败: ' + ((e as any)?.message || '未知错误'));
    }
  };

  const subscribeFolder = async (folderId: string, localPath: string) => {
    try {
      await invoke('subscribe_shared_folder', { folder_id: folderId, local_path: localPath });
      await loadFolders();
    } catch (e) {
      console.error('订阅共享文件夹失败:', e);
      alert('订阅共享文件夹失败: ' + ((e as any)?.message || '未知错误'));
    }
  };

  useEffect(() => {
    loadFolders();
    const unlisten = listen('file-sync', () => {
      loadFolders();
    });
    return () => { unlisten.then(f => f()); };
  }, []);

  return { myFolders, remoteFolders, createFolder, subscribeFolder, refresh: loadFolders };
}
