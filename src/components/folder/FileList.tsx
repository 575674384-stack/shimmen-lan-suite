import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { FileText, Folder, HardDrive, Trash2 } from 'lucide-react';
import type { SharedFolder } from '../../types';

interface FileInfo {
  name: string;
  is_dir: boolean;
  size: number;
  modified: number;
}

interface FileListProps {
  folder: SharedFolder | null;
  isMine: boolean;
  onDelete?: () => void;
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '-';
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
}

export default function FileList({ folder, isMine, onDelete }: FileListProps) {
  const [files, setFiles] = useState<FileInfo[]>([]);

  useEffect(() => {
    if (folder?.local_path) {
      invoke<FileInfo[]>('list_folder_files', { path: folder.local_path })
        .then(setFiles)
        .catch(() => setFiles([]));
    }
  }, [folder?.local_path]);

  if (!folder) {
    return (
      <div className="flex-1 flex items-center justify-center text-text-secondary">
        <div className="text-center">
          <div className="text-4xl mb-2">📁</div>
          <p className="text-sm">选择一个文件夹查看内容</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 flex flex-col bg-surface">
      <div className="h-14 border-b border-border flex items-center justify-between px-4">
        <div>
          <div className="font-medium text-text-primary">{folder.name}</div>
          <div className="text-xs text-text-secondary">{isMine ? '我的共享' : `${folder.owner_name} 的共享`}</div>
        </div>
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2 text-xs text-text-secondary">
            <HardDrive size={14} />
            {folder.local_path}
          </div>
          {isMine && onDelete && (
            <button
              onClick={onDelete}
              className="p-1.5 text-text-secondary hover:text-red-500 hover:bg-red-50 rounded-lg transition-colors"
              title="取消共享"
            >
              <Trash2 size={14} />
            </button>
          )}
        </div>
      </div>

      <div className="flex-1 overflow-auto p-4">
        {files.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-text-secondary">
            <div className="text-4xl mb-2">📂</div>
            <p className="text-sm">文件夹为空</p>
          </div>
        ) : (
          <div className="space-y-1">
            {files.map((file) => (
              <div
                key={file.name}
                className="flex items-center gap-3 p-3 hover:bg-background rounded-xl transition-colors cursor-pointer"
              >
                {file.is_dir ? (
                  <Folder size={20} className="text-yellow-500 shrink-0" />
                ) : (
                  <FileText size={20} className="text-blue-500 shrink-0" />
                )}
                <div className="flex-1 min-w-0">
                  <div className="text-sm text-text-primary truncate">{file.name}</div>
                </div>
                {!file.is_dir && (
                  <div className="text-xs text-text-secondary shrink-0">{formatFileSize(file.size)}</div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
