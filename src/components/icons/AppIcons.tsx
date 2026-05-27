interface IconProps {
  className?: string;
  size?: number;
}

export function UsersIcon({ className = '', size = 24 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" className={className}>
      <circle cx="12" cy="8" r="4" fill="url(#users-grad1)" />
      <path d="M4 20c0-4 3.5-6 8-6s8 2 8 6" fill="url(#users-grad2)" />
      <defs>
        <linearGradient id="users-grad1" x1="8" y1="4" x2="16" y2="12" gradientUnits="userSpaceOnUse">
          <stop stopColor="#0ea5e9" />
          <stop offset="1" stopColor="#6366f1" />
        </linearGradient>
        <linearGradient id="users-grad2" x1="4" y1="14" x2="20" y2="20" gradientUnits="userSpaceOnUse">
          <stop stopColor="#0ea5e9" stopOpacity="0.3" />
          <stop offset="1" stopColor="#6366f1" stopOpacity="0.6" />
        </linearGradient>
      </defs>
    </svg>
  );
}

export function ChatIcon({ className = '', size = 24 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" className={className}>
      <rect x="2" y="4" width="20" height="16" rx="6" fill="url(#chat-grad1)" />
      <circle cx="8" cy="11" r="1.5" fill="white" />
      <circle cx="12" cy="11" r="1.5" fill="white" />
      <circle cx="16" cy="11" r="1.5" fill="white" />
      <path d="M9 16l3 3 3-3" stroke="white" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
      <defs>
        <linearGradient id="chat-grad1" x1="2" y1="4" x2="22" y2="20" gradientUnits="userSpaceOnUse">
          <stop stopColor="#0ea5e9" />
          <stop offset="1" stopColor="#06b6d4" />
        </linearGradient>
      </defs>
    </svg>
  );
}

export function FolderIcon({ className = '', size = 24 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" className={className}>
      <path d="M3 6a2 2 0 012-2h4l2 2h8a2 2 0 012 2v10a2 2 0 01-2 2H5a2 2 0 01-2-2V6z" fill="url(#folder-grad1)" />
      <path d="M3 6a2 2 0 012-2h4l2 2h8a2 2 0 012 2v2H3V6z" fill="url(#folder-grad2)" />
      <circle cx="18" cy="16" r="3" fill="#fbbf24" />
      <path d="M17 16l1 1 2-2" stroke="white" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round" />
      <defs>
        <linearGradient id="folder-grad1" x1="3" y1="4" x2="21" y2="20" gradientUnits="userSpaceOnUse">
          <stop stopColor="#f59e0b" />
          <stop offset="1" stopColor="#f97316" />
        </linearGradient>
        <linearGradient id="folder-grad2" x1="3" y1="4" x2="21" y2="10" gradientUnits="userSpaceOnUse">
          <stop stopColor="#fbbf24" />
          <stop offset="1" stopColor="#f59e0b" />
        </linearGradient>
      </defs>
    </svg>
  );
}

export function VaultIcon({ className = '', size = 24 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" className={className}>
      <rect x="4" y="8" width="16" height="12" rx="3" fill="url(#vault-grad1)" />
      <circle cx="12" cy="14" r="3" fill="url(#vault-grad2)" />
      <rect x="10" y="13" width="4" height="2" rx="1" fill="white" />
      <path d="M12 4v4" stroke="url(#vault-grad1)" strokeWidth="2" strokeLinecap="round" />
      <circle cx="12" cy="4" r="2" fill="url(#vault-grad1)" />
      <defs>
        <linearGradient id="vault-grad1" x1="4" y1="4" x2="20" y2="20" gradientUnits="userSpaceOnUse">
          <stop stopColor="#8b5cf6" />
          <stop offset="1" stopColor="#a855f7" />
        </linearGradient>
        <linearGradient id="vault-grad2" x1="9" y1="11" x2="15" y2="17" gradientUnits="userSpaceOnUse">
          <stop stopColor="#fbbf24" />
          <stop offset="1" stopColor="#f59e0b" />
        </linearGradient>
      </defs>
    </svg>
  );
}

export function AnnouncementIcon({ className = '', size = 24 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" className={className}>
      <rect x="3" y="5" width="14" height="16" rx="3" fill="url(#ann-grad1)" />
      <rect x="6" y="9" width="8" height="1.5" rx="0.75" fill="white" opacity="0.6" />
      <rect x="6" y="13" width="6" height="1.5" rx="0.75" fill="white" opacity="0.6" />
      <path d="M17 8l4-2v12l-4-2V8z" fill="url(#ann-grad2)" />
      <circle cx="17" cy="14" r="1.5" fill="#ef4444" />
      <defs>
        <linearGradient id="ann-grad1" x1="3" y1="5" x2="17" y2="21" gradientUnits="userSpaceOnUse">
          <stop stopColor="#10b981" />
          <stop offset="1" stopColor="#059669" />
        </linearGradient>
        <linearGradient id="ann-grad2" x1="17" y1="6" x2="21" y2="18" gradientUnits="userSpaceOnUse">
          <stop stopColor="#f59e0b" />
          <stop offset="1" stopColor="#f97316" />
        </linearGradient>
      </defs>
    </svg>
  );
}

export function BoardIcon({ className = '', size = 24 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" className={className}>
      <rect x="3" y="3" width="7" height="7" rx="2" fill="url(#board-grad1)" />
      <rect x="14" y="3" width="7" height="7" rx="2" fill="url(#board-grad2)" />
      <rect x="3" y="14" width="7" height="7" rx="2" fill="url(#board-grad3)" />
      <rect x="14" y="14" width="7" height="7" rx="2" fill="url(#board-grad4)" />
      <defs>
        <linearGradient id="board-grad1" x1="3" y1="3" x2="10" y2="10" gradientUnits="userSpaceOnUse">
          <stop stopColor="#0ea5e9" />
          <stop offset="1" stopColor="#38bdf8" />
        </linearGradient>
        <linearGradient id="board-grad2" x1="14" y1="3" x2="21" y2="10" gradientUnits="userSpaceOnUse">
          <stop stopColor="#8b5cf6" />
          <stop offset="1" stopColor="#a78bfa" />
        </linearGradient>
        <linearGradient id="board-grad3" x1="3" y1="14" x2="10" y2="21" gradientUnits="userSpaceOnUse">
          <stop stopColor="#10b981" />
          <stop offset="1" stopColor="#34d399" />
        </linearGradient>
        <linearGradient id="board-grad4" x1="14" y1="14" x2="21" y2="21" gradientUnits="userSpaceOnUse">
          <stop stopColor="#f59e0b" />
          <stop offset="1" stopColor="#fbbf24" />
        </linearGradient>
      </defs>
    </svg>
  );
}

export function ToolsIcon({ className = '', size = 24 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" className={className}>
      <rect x="3" y="3" width="8" height="8" rx="2" fill="url(#tools-grad1)" />
      <rect x="13" y="3" width="8" height="8" rx="2" fill="url(#tools-grad2)" />
      <rect x="3" y="13" width="8" height="8" rx="2" fill="url(#tools-grad3)" />
      <circle cx="17" cy="17" r="4" fill="url(#tools-grad4)" />
      <path d="M15 17h4M17 15v4" stroke="white" strokeWidth="1.5" strokeLinecap="round" />
      <defs>
        <linearGradient id="tools-grad1" x1="3" y1="3" x2="11" y2="11" gradientUnits="userSpaceOnUse">
          <stop stopColor="#0ea5e9" />
          <stop offset="1" stopColor="#38bdf8" />
        </linearGradient>
        <linearGradient id="tools-grad2" x1="13" y1="3" x2="21" y2="11" gradientUnits="userSpaceOnUse">
          <stop stopColor="#8b5cf6" />
          <stop offset="1" stopColor="#a78bfa" />
        </linearGradient>
        <linearGradient id="tools-grad3" x1="3" y1="13" x2="11" y2="21" gradientUnits="userSpaceOnUse">
          <stop stopColor="#10b981" />
          <stop offset="1" stopColor="#34d399" />
        </linearGradient>
        <linearGradient id="tools-grad4" x1="13" y1="13" x2="21" y2="21" gradientUnits="userSpaceOnUse">
          <stop stopColor="#f59e0b" />
          <stop offset="1" stopColor="#fbbf24" />
        </linearGradient>
      </defs>
    </svg>
  );
}

export function ScreenShareIcon({ className = '', size = 24 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" className={className}>
      <rect x="2" y="4" width="20" height="14" rx="3" fill="url(#screen-grad1)" />
      <rect x="4" y="6" width="16" height="10" rx="1.5" fill="#1e293b" opacity="0.3" />
      <circle cx="12" cy="11" r="3" fill="url(#screen-grad2)" />
      <polygon points="11,10 11,12 13,11" fill="white" />
      <rect x="9" y="19" width="6" height="2" rx="1" fill="url(#screen-grad1)" />
      <defs>
        <linearGradient id="screen-grad1" x1="2" y1="4" x2="22" y2="18" gradientUnits="userSpaceOnUse">
          <stop stopColor="#ef4444" />
          <stop offset="1" stopColor="#f97316" />
        </linearGradient>
        <linearGradient id="screen-grad2" x1="9" y1="8" x2="15" y2="14" gradientUnits="userSpaceOnUse">
          <stop stopColor="#ef4444" stopOpacity="0.8" />
          <stop offset="1" stopColor="#f97316" stopOpacity="0.8" />
        </linearGradient>
      </defs>
    </svg>
  );
}
