import type { ReactNode } from 'react';

export interface StripMember {
  id: string;
  name: string;
  platform: string;
  activity: string;
  avatar: ReactNode;
  portrait?: ReactNode;
}

export function MemberStrip({ members, selectedId, onSelect }: {
  members: StripMember[];
  selectedId: string;
  onSelect: (id: string) => void;
}) {
  if (!members.length) return null;
  return (
    <div className="ui-members">
      {members.map((member) => (
        <button key={member.id} type="button" className="ui-member" onClick={() => onSelect(member.id)} aria-pressed={selectedId === member.id} title={`${member.platform} · ${member.activity}`}>
          <span className="ui-member-avatar">
            {member.portrait ?? (Array.from(member.name.trim())[0] || '#')}
            <span className="ui-member-badge" aria-hidden="true">{member.avatar}</span>
          </span>
          <span>{member.name}</span>
        </button>
      ))}
    </div>
  );
}
