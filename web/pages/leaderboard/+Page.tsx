import { useState } from "react";
import { IconTrophy, IconCalendar } from "@tabler/icons-react";
import { mockGlobalData, mockMonthlyData, type LeaderboardEntry } from "./mockData";

function LeaderboardTable({ data }: { data: LeaderboardEntry[] }) {
  return (
    <table className="w-full border-separate border-spacing-y-2">
      <tbody>
        {data.map((entry) => (
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
              <span className="text-yellow-300 font-[600] text-lg">{entry.score.toLocaleString()}</span>
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

const tabButtonClass = (isActive: boolean) =>
  `inline-flex items-center gap-1 px-3 py-1 rounded border ${
    isActive ? "border-yellow-400/40 text-yellow-300" : "border-transparent text-gray-300 hover:text-yellow-200"
  }`;

export default function Page() {
  const [activeTab, setActiveTab] = useState<"global" | "monthly">("global");

  return (
    <div className="page-container">
      <div className="space-y-6">
        <div className="card">
          <div className="flex gap-2 border-b border-yellow-400/20 pb-2 mb-4">
            <button onClick={() => setActiveTab("global")} className={tabButtonClass(activeTab === "global")}>
              <IconTrophy size={16} />
              Global
            </button>
            <button onClick={() => setActiveTab("monthly")} className={tabButtonClass(activeTab === "monthly")}>
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
