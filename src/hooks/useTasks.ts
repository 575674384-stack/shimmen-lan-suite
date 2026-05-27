import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { Task } from '../types';

export function useTasks() {
  const [tasks, setTasks] = useState<Task[]>([]);

  const load = async () => {
    try {
      const data = await invoke<Task[]>('get_tasks');
      setTasks(data);
    } catch (e) {
      console.error('加载任务失败:', e);
    }
  };

  const saveTask = async (task: Task) => {
    try {
      await invoke('save_task', { task });
      await load();
    } catch (e) {
      console.error('保存任务失败:', e);
      alert('保存任务失败: ' + ((e as any)?.message || '未知错误'));
    }
  };

  const deleteTask = async (id: string) => {
    try {
      await invoke('delete_task', { id });
      await load();
    } catch (e) {
      console.error('删除任务失败:', e);
      alert('删除任务失败: ' + ((e as any)?.message || '未知错误'));
    }
  };

  const updateStatus = async (id: string, status: Task['status']) => {
    try {
      await invoke('update_task_status', { id, status });
      await load();
    } catch (e) {
      console.error('更新任务状态失败:', e);
      alert('更新任务状态失败: ' + ((e as any)?.message || '未知错误'));
    }
  };

  useEffect(() => {
    load();
    const unlisten = listen('network-message', (event: any) => {
      // 只对 tasks 相关的 StateSync 触发刷新，避免所有网络包都 reload
      const msg = event.payload?.message;
      if (msg?.type === 'StateSync' && msg?.payload?.table === 'tasks') {
        load();
      }
    });
    return () => { unlisten.then(f => f()); };
  }, []);

  return { tasks, load, saveTask, deleteTask, updateStatus };
}
