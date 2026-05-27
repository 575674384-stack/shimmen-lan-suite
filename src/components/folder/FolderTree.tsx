import { FolderPlus, FolderSync, Monitor } from 'lucide-react';
import type { SharedFolder } from '../../types';

interface FolderTreeProps {
  myFolders: SharedFolder[];
  remoteFolders: SharedFolder[];
  selectedId: string | null;
  onSelect: (folder: SharedFolder) => void;
  onCreate: () => void;
}

export default function FolderTree({ myFolders, remoteFolders, selectedId, onSelect, onCreate }: FolderTreeProps) {
  return (
    <div className="w-72 bg-background border-r border-border flex flex-col">
      <div className="p-4 border-b border-border">
        <div className="flex items-center justify-between">
          <h2 className="font-bold text-text-primary flex items-center gap-2">
            <FolderSync size={18} className="text-primary" />
            共享文件夹
          </h2>
          <button
            onClick={onCreate}
            className="p-1.5 text-primary hover:bg-primary-light rounded-lg transition-colors"
          >
            <FolderPlus size={16} />
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-auto p-2">
        {/* 我的共享 */}
        <div className="mb-4">
          <div className="text-xs font-medium text-text-secondary uppercase tracking-wider px-2 mb-1">我的共享</div>
          {myFolders.length === 0 && (
            <div className="px-2 py-3 text-xs text-text-secondary">点击 + 创建共享文件夹</div>
          )}
          {myFolders.map((folder) => (
            <button
              key={folder.id}
              onClick={() => onSelect(folder)}
              className={`w-full flex items-center gap-2 p-2 rounded-lg text-left transition-colors ${
                selectedId === folder.id ? 'bg-surface shadow-sm text-primary-dark' : 'hover:bg-surface text-text-primary'
              }`}
            >
              <FolderSync size={16} className={folder.sync_status === 'syncing' ? 'text-green-500' : 'text-text-secondary'} />
              <div className="flex-1 min-w-0">
                <div className="text-sm font-medium truncate">{folder.name}</div>
                <div className="text-[10px] text-text-secondary truncate">{folder.local_path}</div>
              </div>
            </button>
          ))}
        </div>

        {/* 在线用户的共享 */}
        <div>
          <div className="text-xs font-medium text-text-secondary uppercase tracking-wider px-2 mb-1">在线用户</div>
          {remoteFolders.length === 0 && (
            <div className="px-2 py-3 text-xs text-text-secondary">暂无其他用户共享文件夹</div>
          )}
          {remoteFolders.map((folder) => (
            <button
              key={folder.id}
              onClick={() => onSelect(folder)}
              className={`w-full flex items-center gap-2 p-2 rounded-lg text-left transition-colors ${
                selectedId === folder.id ? 'bg-surface shadow-sm text-primary-dark' : 'hover:bg-surface text-text-primary'
              }`}
            >
              <Monitor size={14} className="text-text-secondary" />
              <div className="flex-1 min-w-0">
                <div className="text-sm font-medium truncate">{folder.name}</div>
                <div className="text-[10px] text-text-secondary truncate">{folder.owner_name} · {folder.ip || ''}</div>
              </div>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
