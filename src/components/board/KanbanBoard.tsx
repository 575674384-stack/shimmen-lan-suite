import { useState } from 'react';
import { DndContext, DragEndEvent, PointerSensor, useSensor } from '@dnd-kit/core';
import { Plus, Layout, FolderOpen } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import KanbanColumn from './KanbanColumn';
import { useTasks } from '../../hooks/useTasks';
import type { Task, SharedFolder } from '../../types';

export default function KanbanBoard() {
  const { tasks, saveTask, deleteTask, updateStatus } = useTasks();
  const [showModal, setShowModal] = useState(false);
  const [editing, setEditing] = useState<Task | null>(null);
  const [form, setForm] = useState<Partial<Task>>({});
  const [sharedFolders, setSharedFolders] = useState<SharedFolder[]>([]);
  const [archiveFolderId, setArchiveFolderId] = useState('');

  const pointerSensor = useSensor(PointerSensor, {
    activationConstraint: {
      distance: 8,
    },
  });

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (over && active.id !== over.id) {
      const newStatus = over.id as Task['status'];
      updateStatus(active.id as string, newStatus);
    }
  };

  const handleSave = async () => {
    if (!form.title?.trim()) return;
    await saveTask({
      id: editing?.id || crypto.randomUUID(),
      title: form.title,
      project: form.project || '',
      deadline: form.deadline || null,
      contact: form.contact || '',
      priority: form.priority || 'medium',
      description: form.description || '',
      status: editing?.status || 'todo',
      creator_id: '',
      assignee_id: null,
      is_team_visible: form.is_team_visible || false,
      attached_files: form.attached_files || [],
      archived_to_folder_id: form.archived_to_folder_id,
    });
    setShowModal(false);
    setEditing(null);
    setForm({});
  };

  const loadFolders = async () => {
    try {
      const data = await invoke<SharedFolder[]>('get_my_shared_folders');
      setSharedFolders(data);
    } catch (e) {
      console.error('加载共享文件夹失败:', e);
    }
  };

  const handleArchive = async () => {
    if (!editing?.id || !archiveFolderId) return;
    try {
      await invoke('archive_task', { taskId: editing.id, folderId: archiveFolderId });
      alert('归档成功！');
      setShowModal(false);
      setEditing(null);
      setForm({});
    } catch (err) {
      console.error('归档失败:', err);
      alert('归档失败: ' + (err instanceof Error ? err.message : String(err)));
    }
  };

  const columns = [
    { id: 'todo', title: '待处理', color: 'bg-gray-400' },
    { id: 'doing', title: '处理中', color: 'bg-blue-500' },
    { id: 'done', title: '已完成', color: 'bg-green-500' },
  ];

  return (
    <div className="h-full flex flex-col bg-background">
      <div className="h-16 bg-surface border-b border-border flex items-center justify-between px-5">
        <div className="flex items-center gap-2.5">
          <Layout size={22} className="text-primary" />
          <h2 className="font-bold text-base text-text-primary">我的工作台</h2>
        </div>
        <button onClick={() => { setShowModal(true); setEditing(null); setForm({}); }} className="flex items-center gap-1.5 px-4 py-2 bg-primary text-white text-base rounded-xl hover:bg-primary-dark transition-colors">
          <Plus size={18} /> 新建任务
        </button>
      </div>

      <div className="flex-1 overflow-auto p-5">
        <DndContext sensors={[pointerSensor]} onDragEnd={handleDragEnd}>
          <div className="flex gap-4 h-full">
            {columns.map((col) => (
              <KanbanColumn
                key={col.id}
                id={col.id}
                title={col.title}
                color={col.color}
                tasks={tasks.filter((t) => t.status === col.id)}
                onTaskClick={(task) => { setEditing(task); setForm(task); setShowModal(true); }}
              />
            ))}
          </div>
        </DndContext>
      </div>

      {showModal && (
        <div className="fixed inset-0 bg-black/30 flex items-center justify-center z-50">
          <div className="bg-surface rounded-2xl p-6 w-[520px] shadow-xl max-h-[90vh] overflow-auto">
            <h3 className="font-bold text-xl text-text-primary mb-5">{editing ? '编辑' : '新建'}任务</h3>
            <div className="space-y-4">
              <input placeholder="任务标题" value={form.title || ''} onChange={(e) => setForm({ ...form, title: e.target.value })} className="w-full px-4 py-2.5 bg-background border border-border rounded-xl text-base focus:outline-none focus:ring-2 focus:ring-primary/20" />
              <input placeholder="项目名称" value={form.project || ''} onChange={(e) => setForm({ ...form, project: e.target.value })} className="w-full px-4 py-2.5 bg-background border border-border rounded-xl text-base focus:outline-none focus:ring-2 focus:ring-primary/20" />
              <div className="flex gap-3">
                <input type="date" placeholder="截止日期" value={form.deadline || ''} onChange={(e) => setForm({ ...form, deadline: e.target.value })} className="flex-1 px-4 py-2.5 bg-background border border-border rounded-xl text-base focus:outline-none focus:ring-2 focus:ring-primary/20" />
                <select value={form.priority || 'medium'} onChange={(e) => setForm({ ...form, priority: e.target.value as Task['priority'] })} className="px-4 py-2.5 bg-background border border-border rounded-xl text-base focus:outline-none focus:ring-2 focus:ring-primary/20">
                  <option value="low">低优先级</option>
                  <option value="medium">中优先级</option>
                  <option value="high">高优先级</option>
                </select>
              </div>
              <input placeholder="联系人" value={form.contact || ''} onChange={(e) => setForm({ ...form, contact: e.target.value })} className="w-full px-4 py-2.5 bg-background border border-border rounded-xl text-base focus:outline-none focus:ring-2 focus:ring-primary/20" />
              <textarea placeholder="描述" value={form.description || ''} onChange={(e) => setForm({ ...form, description: e.target.value })} rows={3} className="w-full px-4 py-2.5 bg-background border border-border rounded-xl text-base focus:outline-none focus:ring-2 focus:ring-primary/20 resize-none" />
              <label className="flex items-center gap-2 text-base text-text-secondary">
                <input type="checkbox" checked={form.is_team_visible || false} onChange={(e) => setForm({ ...form, is_team_visible: e.target.checked })} className="rounded" />
                对团队可见
              </label>
              {editing?.status === 'done' && (
                <div className="space-y-3">
                  <label className="text-base text-text-secondary">附加文件</label>
                  <div className="flex flex-wrap gap-2">
                    {(form.attached_files || []).map((file, idx) => (
                      <span key={idx} className="text-sm bg-background px-3 py-1.5 rounded-lg text-text-secondary">{file}</span>
                    ))}
                    <button
                      onClick={async () => {
                        try {
                          const path = await invoke<string | null>('select_file');
                          if (path) {
                            setForm({ ...form, attached_files: [...(form.attached_files || []), path] });
                          }
                        } catch (e) {
                          console.error('选择文件失败:', e);
                        }
                      }}
                      className="text-sm px-3 py-1.5 border border-dashed border-border rounded-lg text-text-secondary hover:text-primary hover:border-primary transition-colors"
                    >
                      + 添加文件
                    </button>
                    <button
                      onClick={async () => {
                        try {
                          const path = await invoke<string | null>('select_folder');
                          if (path) {
                            setForm({ ...form, attached_files: [...(form.attached_files || []), path] });
                          }
                        } catch (e) {
                          console.error('选择文件夹失败:', e);
                        }
                      }}
                      className="text-sm px-3 py-1.5 border border-dashed border-border rounded-lg text-text-secondary hover:text-primary hover:border-primary transition-colors flex items-center gap-1"
                    >
                      <FolderOpen size={14} /> 添加文件夹
                    </button>
                  </div>
                  <div className="pt-3 border-t border-border">
                    <label className="text-base text-text-secondary block mb-2">归档到共享文件夹</label>
                    <div className="flex gap-3">
                      <select
                        value={archiveFolderId}
                        onClick={() => { if (sharedFolders.length === 0) loadFolders(); }}
                        onChange={(e) => setArchiveFolderId(e.target.value)}
                        className="flex-1 px-4 py-2.5 bg-background border border-border rounded-xl text-base focus:outline-none"
                      >
                        <option value="">选择共享文件夹...</option>
                        {sharedFolders.map((f) => (
                          <option key={f.id} value={f.id}>{f.name}</option>
                        ))}
                      </select>
                      <button
                        onClick={handleArchive}
                        disabled={!archiveFolderId}
                        className="px-4 py-2.5 bg-green-500 text-white text-base rounded-xl hover:bg-green-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                      >
                        归档
                      </button>
                    </div>
                  </div>
                </div>
              )}
            </div>
            <div className="flex justify-end gap-3 mt-7">
              {editing && (
                <button onClick={() => { deleteTask(editing.id); setShowModal(false); }} className="px-5 py-2.5 text-base text-red-500 hover:bg-red-50 rounded-xl">删除</button>
              )}
              <button onClick={() => setShowModal(false)} className="px-5 py-2.5 text-base text-text-secondary hover:bg-background rounded-xl">取消</button>
              <button onClick={handleSave} className="px-5 py-2.5 text-base bg-primary text-white rounded-xl hover:bg-primary-dark">保存</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
