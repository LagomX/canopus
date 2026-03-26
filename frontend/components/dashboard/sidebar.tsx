"use client"

import { cn } from "@/lib/utils"
import { BookOpen, Moon, CheckSquare, LayoutDashboard, BarChart2 } from "lucide-react"

type Section = "overview" | "journal" | "sleep" | "tasks" | "reports"

export type { Section }

interface SidebarProps {
  activeSection: Section
  onSectionChange: (section: Section) => void
}

const navItems = [
  { id: "overview" as const, label: "概览", icon: LayoutDashboard },
  { id: "journal" as const, label: "日记", icon: BookOpen },
  { id: "sleep" as const, label: "睡眠", icon: Moon },
  { id: "tasks" as const, label: "任务", icon: CheckSquare },
  { id: "reports" as const, label: "报告", icon: BarChart2 },
]

export function Sidebar({ activeSection, onSectionChange }: SidebarProps) {
  return (
    <aside className="fixed left-0 top-0 z-40 flex h-screen w-64 flex-col border-r border-sidebar-border bg-sidebar">
      <div className="flex h-16 items-center gap-2 border-b border-sidebar-border px-6">
        <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary">
          <span className="text-sm font-bold text-primary-foreground">C</span>
        </div>
        <span className="text-lg font-semibold text-sidebar-foreground">Canopus</span>
      </div>
      
      <nav className="flex-1 space-y-1 p-4">
        {navItems.map((item) => {
          const Icon = item.icon
          const isActive = activeSection === item.id
          return (
            <button
              key={item.id}
              onClick={() => onSectionChange(item.id)}
              className={cn(
                "flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors",
                isActive
                  ? "bg-sidebar-accent text-sidebar-primary"
                  : "text-muted-foreground hover:bg-sidebar-accent hover:text-sidebar-foreground"
              )}
            >
              <Icon className="h-4 w-4" />
              {item.label}
            </button>
          )
        })}
      </nav>
      
      <div className="border-t border-sidebar-border p-4">
        <div className="rounded-lg bg-sidebar-accent p-3">
          <p className="text-xs text-muted-foreground">
            记录每一天，成就更好的自己
          </p>
        </div>
      </div>
    </aside>
  )
}
