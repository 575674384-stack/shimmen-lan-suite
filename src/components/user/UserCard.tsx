import { Monitor, Circle } from 'lucide-react';
import type { User } from '../../types';

interface UserCardProps {
  user: User;
  onClick: () => void;
}

const gradients = [
  'from-sky-400 to-blue-500',
  'from-emerald-400 to-teal-500',
  'from-orange-400 to-red-400',
  'from-purple-400 to-pink-500',
  'from-indigo-400 to-purple-500',
];

export default function UserCard({ user, onClick }: UserCardProps) {
  const colorIndex = user.id.charCodeAt(0) % gradients.length;
  const gradient = gradients[colorIndex];

  return (
    <button
      onClick={onClick}
      className="w-full flex items-center gap-3 p-3 rounded-xl hover:bg-surface hover:shadow-sm transition-all text-left group"
    >
      <div className="relative">
        <div className={`w-10 h-10 bg-gradient-to-br ${gradient} rounded-full flex items-center justify-center text-white font-medium text-sm`}>
          {user.username.slice(0, 2)}
        </div>
        <Circle
          size={10}
          className={`absolute -bottom-0.5 -right-0.5 fill-current ${
            user.status === 'online' ? 'text-green-500' : 'text-text-secondary'
          }`}
          strokeWidth={0}
        />
      </div>
      <div className="flex-1 min-w-0">
        <div className="font-medium text-text-primary text-sm truncate">{user.username}</div>
        <div className="text-xs text-text-secondary flex items-center gap-1">
          <Monitor size={10} />
          {user.ip}
        </div>
      </div>
      <div className="text-[10px] text-text-secondary opacity-0 group-hover:opacity-100 transition-opacity">
        v{user.version}
      </div>
    </button>
  );
}
