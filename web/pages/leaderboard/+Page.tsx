import { useState } from "react";
import { IconTrophy, IconCalendar } from "@tabler/icons-react";

interface LeaderboardEntry {
  id: number;
  rank: number;
  name: string;
  score: number;
  duration: string;
  levelCount: number;
  submittedAt: string;
  avatar?: string;
}

const mockGlobalData: LeaderboardEntry[] = [
  {
    id: 1,
    rank: 1,
    name: "PacMaster2024",
    score: 125000,
    duration: "45:32",
    levelCount: 12,
    submittedAt: "2 hours ago",
    avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=PacMaster2024",
  },
  {
    id: 2,
    rank: 2,
    name: "GhostHunter",
    score: 118750,
    duration: "42:18",
    levelCount: 11,
    submittedAt: "5 hours ago",
    avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=GhostHunter",
  },
  {
    id: 3,
    rank: 3,
    name: "DotCollector",
    score: 112500,
    duration: "38:45",
    levelCount: 10,
    submittedAt: "1 day ago",
    avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=DotCollector",
  },
  {
    id: 4,
    rank: 4,
    name: "MazeRunner",
    score: 108900,
    duration: "41:12",
    levelCount: 10,
    submittedAt: "2 days ago",
    avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=MazeRunner",
  },
  {
    id: 5,
    rank: 5,
    name: "PowerPellet",
    score: 102300,
    duration: "36:28",
    levelCount: 9,
    submittedAt: "3 days ago",
    avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=PowerPellet",
  },
  {
    id: 6,
    rank: 6,
    name: "CherryPicker",
    score: 98750,
    duration: "39:15",
    levelCount: 9,
    submittedAt: "4 days ago",
    avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=CherryPicker",
  },
  {
    id: 7,
    rank: 7,
    name: "BlinkyBeater",
    score: 94500,
    duration: "35:42",
    levelCount: 8,
    submittedAt: "5 days ago",
    avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=BlinkyBeater",
  },
  {
    id: 8,
    rank: 8,
    name: "PinkyPac",
    score: 91200,
    duration: "37:55",
    levelCount: 8,
    submittedAt: "1 week ago",
    avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=PinkyPac",
  },
  {
    id: 9,
    rank: 9,
    name: "InkyDestroyer",
    score: 88800,
    duration: "34:18",
    levelCount: 8,
    submittedAt: "1 week ago",
    avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=InkyDestroyer",
  },
  {
    id: 10,
    rank: 10,
    name: "ClydeChaser",
    score: 85600,
    duration: "33:45",
    levelCount: 7,
    submittedAt: "1 week ago",
    avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=ClydeChaser",
  },
];

const mockMonthlyData: LeaderboardEntry[] = [
  {
    id: 1,
    rank: 1,
    name: "JanuaryChamp",
    score: 115000,
    duration: "43:22",
    levelCount: 11,
    submittedAt: "1 day ago",
    avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=JanuaryChamp",
  },
  {
    id: 2,
    rank: 2,
    name: "NewYearPac",
    score: 108500,
    duration: "40:15",
    levelCount: 10,
    submittedAt: "3 days ago",
    avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=NewYearPac",
  },
  {
    id: 3,
    rank: 3,
    name: "WinterWarrior",
    score: 102000,
    duration: "38:30",
    levelCount: 10,
    submittedAt: "5 days ago",
    avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=WinterWarrior",
  },
  {
    id: 4,
    rank: 4,
    name: "FrostyPac",
    score: 98500,
    duration: "37:45",
    levelCount: 9,
    submittedAt: "1 week ago",
    avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=FrostyPac",
  },
  {
    id: 5,
    rank: 5,
    name: "IceBreaker",
    score: 95200,
    duration: "36:12",
    levelCount: 9,
    submittedAt: "1 week ago",
    avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=IceBreaker",
  },
  {
    id: 6,
    rank: 6,
    name: "SnowPac",
    score: 91800,
    duration: "35:28",
    levelCount: 8,
    submittedAt: "2 weeks ago",
    avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=SnowPac",
  },
  {
    id: 7,
    rank: 7,
    name: "BlizzardBeast",
    score: 88500,
    duration: "34:15",
    levelCount: 8,
    submittedAt: "2 weeks ago",
    avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=BlizzardBeast",
  },
  {
    id: 8,
    rank: 8,
    name: "ColdSnap",
    score: 85200,
    duration: "33:42",
    levelCount: 8,
    submittedAt: "3 weeks ago",
    avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=ColdSnap",
  },
  {
    id: 9,
    rank: 9,
    name: "FrozenFury",
    score: 81900,
    duration: "32:55",
    levelCount: 7,
    submittedAt: "3 weeks ago",
    avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=FrozenFury",
  },
  {
    id: 10,
    rank: 10,
    name: "ArcticAce",
    score: 78600,
    duration: "31:18",
    levelCount: 7,
    submittedAt: "4 weeks ago",
    avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=ArcticAce",
  },
];

function LeaderboardTable({ data }: { data: LeaderboardEntry[] }) {
  return (
    <table className="w-full border-separate border-spacing-y-2">
      <tbody>
        {data.map((entry, entryIndex) => (
          <tr key={entry.id} className="bg-black">
            <td className="py-2">
              <div className="flex items-center gap-2">
                <img src={entry.avatar} alt={entry.name} className="w-9 h-9 rounded-sm" loading="lazy" />
                <div className="flex flex-col">
                  <span className="text-yellow-400 font-semibold text-lg">{entry.name}</span>
                  <span className="text-xs text-gray-400">{entry.submittedAt}</span>
                </div>
              </div>
            </td>
            <td className="py-2">
              <span className="text-yellow-300 font-[600] text-lg">
                {entry.score.toLocaleString()}
              </span>
            </td>
            <td className="py-2">
              <span className="text-gray-300">{entry.duration}</span>
            </td>
            <td className="py-2">Level {entry.levelCount}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

export default function Page() {
  const [activeTab, setActiveTab] = useState<"global" | "monthly">("global");

  return (
    <div className="mx-auto max-w-3xl py-8 px-4">
      <div className="space-y-6">
        <div className="border border-yellow-400/20 rounded-md bg-transparent p-6 shadow-[0_4px_20px_rgba(250,204,21,0.08)]">
          <div className="flex gap-2 border-b border-yellow-400/20 pb-2 mb-4">
            <button
              onClick={() => setActiveTab("global")}
              className={
                activeTab === "global"
                  ? "inline-flex items-center gap-1 px-3 py-1 rounded border border-yellow-400/40 text-yellow-300"
                  : "inline-flex items-center gap-1 px-3 py-1 rounded border border-transparent text-gray-300 hover:text-yellow-200"
              }
            >
              <IconTrophy size={16} />
              Global
            </button>
            <button
              onClick={() => setActiveTab("monthly")}
              className={
                activeTab === "monthly"
                  ? "inline-flex items-center gap-1 px-3 py-1 rounded border border-yellow-400/40 text-yellow-300"
                  : "inline-flex items-center gap-1 px-3 py-1 rounded border border-transparent text-gray-300 hover:text-yellow-200"
              }
            >
              <IconCalendar size={16} />
              Monthly
            </button>
          </div>

          {activeTab === "global" ? (
            <LeaderboardTable data={mockGlobalData} />
          ) : (
            <LeaderboardTable data={mockMonthlyData} />
          )}
        </div>
      </div>
    </div>
  );
}
