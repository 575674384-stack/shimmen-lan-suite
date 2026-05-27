import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import FolderTree from './FolderTree';
import FileList from './FileList';
import { useFolders } from '../../hooks/useFolders';
import type { SharedFolder } from '../../types';

export default function FolderBrowser() {
  const { myFolders, remoteFolders, createFolder, refresh } = useFolders();
  const [selectedFolder, setSelectedFolder] = useState<SharedFolder | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newFolderName, setNewFolderName] = useState('');
  const [newFolderPath, setNewFolderPath] = useState('');

  const handleCreate = async () => {
    if (!newFolderName.trim() || !newFolderPath.trim()) return;
    await createFolder(newFolderName.trim(), newFolderPath.trim());
    setShowCreateModal(false);
    setNewFolderName('');
    setNewFolderPath('');
  };

  const handleBrowse = async () => {
    try {
      const selected = await invoke<string | null>('select_folder');
      if (selected) {
        setNewFolderPath(selected);
      }
    } catch (e) {
      console.error('选择文件夹失败:', e);
    }
  };

  const handleDeleteFolder = async (folderId: string) => {
    if (!confirm('确定取消共享此文件夹？')) return;
    try {
      await invoke('delete_shared_folder', { id: folderId });
      await refresh();
      if (selectedFolder?.id === folderId) {
        setSelectedFolder(null);
      }
    } catch (e) {
      console.error('取消共享文件夹失败:', e);
      alert('取消共享文件夹失败: ' + ((e as any)?.message || '未知错误'));
    }
  };

  const isMine = selectedFolder ? myFolders.some((f) => f.id === selectedFolder.id) : false;

  return (
    <div className="h-full flex">
      <FolderTree
        myFolders={myFolders}
        remoteFolders={remoteFolders}
        selectedId={selectedFolder?.id || null}
        onSelect={setSelectedFolder}
        onCreate={() => setShowCreateModal(true)}
      />
      <FileList 
        folder={selectedFolder} 
        isMine={isMine} 
        onDelete={isMine && selectedFolder ? () => handleDeleteFolder(selectedFolder.id) : undefined}
      />

      {showCreateModal && (
        <div className="fixed inset-0 bg-black/30 flex items-center justify-center z-50">
          <div className="bg-surface rounded-2xl p-6 w-96 shadow-xl">
            <h3 className="font-bold text-lg text-text-primary mb-4">创建共享文件夹</h3>
            <div className="space-y-3">
              <div>
                <label className="text-sm text-text-secondary block mb-1">文件夹名称</label>
                <input
                  type="text"
                  value={newFolderName}
                  onChange={(e) => setNewFolderName(e.target.value)}
                  placeholder="如：团队文档"
                  className="w-full px-3 py-2 bg-background border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary/20"
                />
              </div>
              <div>
                <label className="text-sm text-text-secondary block mb-1">本地路径</label>
                <div className="flex items-center gap-2">
                  <input
                    type="text"
                    value={newFolderPath}
                    onChange={(e) => setNewFolderPath(e.target.value)}
                    placeholder="如：C:\\Users\\Documents\\团队文档"
                    className="flex-1 px-3 py-2 bg-background border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary/20"
                  />
                  <button
                    onClick={handleBrowse}
                    className="px-3 py-2 bg-background border border-border rounded-lg text-sm text-text-secondary hover:text-text-primary hover:border-primary transition-colors shrink-0"
                  >
                    浏览...
                  </button>
                </div>
              </div>
            </div>
            <div className="flex justify-end gap-2 mt-6">
              <button
                onClick={() => setShowCreateModal(false)}
                className="px-4 py-2 text-sm text-text-secondary hover:bg-background rounded-lg transition-colors"
              >
                取消
              </button>
              <button
                onClick={handleCreate}
                className="px-4 py-2 text-sm bg-primary text-white rounded-lg hover:bg-primary-dark transition-colors"
              >
                创建
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
