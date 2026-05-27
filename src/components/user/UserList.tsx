import { useState, useEffect, useCallback } from 'react';
import { Users, RefreshCw } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import UserCard from './UserCard';
import TransferWindow from '../chat/TransferWindow';
import type { User } from '../../types';

export default function UserList() {
  const [users, setUsers] = useState<User[]>([]);
  const [selectedUser, setSelectedUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(false);

  const loadUsers = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoke<User[]>('get_online_users_cmd');
      setUsers(data);
    } catch (e) {
      console.error('获取在线用户失败:', e);
      setUsers([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadUsers();
    const timer = setInterval(loadUsers, 3000);
    return () => clearInterval(timer);
  }, [loadUsers]);

  return (
    <div className="h-full flex">
      <div className="w-72 bg-background border-r border-border flex flex-col">
        <div className="p-4 border-b border-border">
          <div className="flex items-center justify-between">
            <h2 className="font-bold text-text-primary flex items-center gap-2">
              <Users size={18} className="text-primary" />
              在线用户
            </h2>
            <button
              onClick={loadUsers}
              disabled={loading}
              className="p-1.5 text-text-secondary hover:text-primary hover:bg-primary-light rounded-lg transition-colors disabled:opacity-50"
            >
              <RefreshCw size={14} className={loading ? 'animate-spin' : ''} />
            </button>
          </div>
          <p className="text-xs text-text-secondary mt-1">
            {users.length > 0 ? `发现 ${users.length} 个在线节点` : '暂无其他在线用户'}
          </p>
        </div>
        <div className="flex-1 overflow-auto p-2 space-y-1">
          {users.length === 0 && (
            <div className="text-center text-text-secondary text-sm py-8">
              <div className="text-3xl mb-2">🔍</div>
              <p>未发现在线用户</p>
              <p className="text-xs mt-1">确保其他设备在同一局域网并运行本软件</p>
            </div>
          )}
          {users.map((user) => (
            <UserCard key={user.id} user={user} onClick={() => setSelectedUser(user)} />
          ))}
        </div>
      </div>

      {selectedUser && (
        <TransferWindow
          user={selectedUser}
          onClose={() => setSelectedUser(null)}
        />
      )}
    </div>
  );
}
