import { Settings } from 'lucide-react';
import { useAvatar } from '../../hooks/useAvatar';
import {
  UsersIcon,
  ChatIcon,
  FolderIcon,
  VaultIcon,
  AnnouncementIcon,
  BoardIcon,
  ScreenShareIcon,
  ToolsIcon,
} from '../icons/AppIcons';

interface SidebarProps {
  activeTab: string;
  onTabChange: (tab: string) => void;
  onOpenSettings: () => void;
}

const tabs = [
  { id: 'users', icon: UsersIcon, label: '用户' },
  { id: 'chat', icon: ChatIcon, label: '群聊' },
  { id: 'folders', icon: FolderIcon, label: '文件夹' },
  { id: 'passwords', icon: VaultIcon, label: '密码' },
  { id: 'announcements', icon: AnnouncementIcon, label: '公告' },
  { id: 'board', icon: BoardIcon, label: '看板' },
  { id: 'screen', icon: ScreenShareIcon, label: '演示' },
  { id: 'tools', icon: ToolsIcon, label: '工具' },
];

const presetMap: Record<string, { emoji: string; gradient: string }> = {
  cat: { emoji: '🐱', gradient: 'from-orange-400 to-pink-500' },
  dog: { emoji: '🐶', gradient: 'from-blue-400 to-purple-500' },
  fox: { emoji: '🦊', gradient: 'from-red-400 to-yellow-500' },
  panda: { emoji: '🐼', gradient: 'from-gray-700 to-gray-300' },
  rabbit: { emoji: '🐰', gradient: 'from-pink-300 to-white' },
  tiger: { emoji: '🐯', gradient: 'from-orange-500 to-yellow-400' },
  frog: { emoji: '🐸', gradient: 'from-green-400 to-emerald-600' },
  octopus: { emoji: '🐙', gradient: 'from-purple-500 to-red-500' },
};

export default function Sidebar({ activeTab, onTabChange, onOpenSettings }: SidebarProps) {
  const { avatarBase64, avatarPreset, username } = useAvatar();

  const renderAvatar = () => {
    if (avatarBase64) {
      return (
        <img
          src={avatarBase64}
          alt="avatar"
          className="w-full h-full object-cover rounded-full"
        />
      );
    }
    if (avatarPreset && presetMap[avatarPreset]) {
      const p = presetMap[avatarPreset];
      return (
        <div className={`w-full h-full bg-gradient-to-br ${p.gradient} rounded-full flex items-center justify-center text-xl`}>
          {p.emoji}
        </div>
      );
    }
    return (
      <div className="w-full h-full bg-primary text-white flex items-center justify-center font-bold text-lg">
        {username.slice(0, 2) || 'S'}
      </div>
    );
  };

  return (
    <div className="w-[88px] bg-surface border-r border-border flex flex-col items-center py-5 select-none shrink-0">
      {/* Avatar */}
      <button
        onClick={onOpenSettings}
        className="w-12 h-12 rounded-xl overflow-hidden mb-6 shadow-sm hover:opacity-90 transition-opacity"
        title="设置"
      >
        {renderAvatar()}
      </button>
      
      {/* 导航图标 */}
      <div className="flex-1 flex flex-col gap-2 w-full px-2">
        {tabs.map((tab) => {
          const Icon = tab.icon;
          const isActive = activeTab === tab.id;
          return (
            <button
              key={tab.id}
              onClick={() => onTabChange(tab.id)}
              className={`relative flex flex-col items-center gap-1.5 py-3 rounded-xl transition-all duration-200 group ${
                isActive
                  ? 'bg-primary-light text-primary'
                  : 'text-text-secondary hover:bg-background hover:text-text-primary'
              }`}
              title={tab.label}
            >
              <Icon size={28} className={isActive ? 'drop-shadow-sm' : 'opacity-80 group-hover:opacity-100'} />
              <span className={`text-xs ${isActive ? 'font-medium' : ''}`}>{tab.label}</span>
            </button>
          );
        })}
      </div>
      
      {/* 底部设置 */}
      <button
        onClick={onOpenSettings}
        className="mt-auto p-3 rounded-xl text-text-secondary hover:bg-background hover:text-text-primary transition-colors"
        title="设置"
      >
        <Settings size={24} strokeWidth={1.5} />
      </button>
    </div>
  );
}
