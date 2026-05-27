import { useDraggable } from '@dnd-kit/core';
import { Calendar, User } from 'lucide-react';
import type { Task } from '../../types';

interface TaskCardProps {
  task: Task;
  onClick: () => void;
}

export default function TaskCard({ task, onClick }: TaskCardProps) {
  const { attributes, listeners, setNodeRef, transform } = useDraggable({
    id: task.id,
    data: task,
  });
  const style = transform ? { transform: `translate3d(${transform.x}px, ${transform.y}px, 0)` } : undefined;

  const priorityColors = {
    low: 'bg-background text-text-secondary',
    medium: 'bg-yellow-100 text-yellow-700',
    high: 'bg-red-100 text-red-700',
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      {...listeners}
      {...attributes}
      onClick={onClick}
      className="bg-surface p-4 rounded-xl shadow-sm border border-border cursor-grab active:cursor-grabbing hover:shadow-md transition-shadow"
    >
      <div className="flex items-start justify-between mb-2">
        <h4 className="font-medium text-base text-text-primary flex-1">{task.title}</h4>
        <span className={`text-xs px-2 py-0.5 rounded-full ${priorityColors[task.priority]}`}>
          {task.priority === 'high' ? '高' : task.priority === 'medium' ? '中' : '低'}
        </span>
      </div>
      <div className="text-sm text-text-secondary mb-2">{task.project}</div>
      <div className="flex items-center gap-3 text-xs text-text-secondary">
        {task.deadline && (
          <span className="flex items-center gap-1">
            <Calendar size={12} />
            {new Date(task.deadline).toLocaleDateString('zh-CN')}
          </span>
        )}
        {task.contact && (
          <span className="flex items-center gap-1">
            <User size={12} />
            {task.contact}
          </span>
        )}
      </div>
    </div>
  );
}
