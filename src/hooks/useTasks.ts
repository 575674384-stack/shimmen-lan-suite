import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { Task } from '../types';

export function useTasks() {
  const [tasks, setTasks] = useState<Task[]>([]);

  const load = async () => {
    const data = await invoke<Task[]>('get_tasks');
    setTasks(data);
  };

  const saveTask = async (task: Task) => {
    await invoke('save_task', { task });
    await load();
  };

  const deleteTask = async (id: string) => {
    await invoke('delete_task', { id });
    await load();
  };

  const updateStatus = async (id: string, status: Task['status']) => {
    await invoke('update_task_status', { id, status });
    await load();
  };

  useEffect(() => { load(); }, []);

  return { tasks, load, saveTask, deleteTask, updateStatus };
}
