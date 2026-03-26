"use client"

import { useState } from "react"
import { Sidebar } from "@/components/dashboard/sidebar"
import type { Section } from "@/components/dashboard/sidebar"
import { Header } from "@/components/dashboard/header"
import { OverviewSection } from "@/components/dashboard/overview-section"
import { JournalSection } from "@/components/dashboard/journal-section"
import { SleepSection } from "@/components/dashboard/sleep-section"
import { TaskSection } from "@/components/dashboard/task-section"
import { ReportSection } from "@/components/dashboard/report-section"

const sectionTitles: Record<Section, string> = {
  overview: "概览",
  journal: "日记",
  sleep: "睡眠",
  tasks: "任务",
  reports: "报告",
}

export default function DashboardPage() {
  const [activeSection, setActiveSection] = useState<Section>("overview")

  return (
    <div className="flex min-h-screen bg-background">
      <Sidebar activeSection={activeSection} onSectionChange={setActiveSection} />

      <div className="flex-1 pl-64">
        <Header />

        <main className="p-6">
          <div className="mb-6">
            <h2 className="text-2xl font-bold text-foreground">{sectionTitles[activeSection]}</h2>
          </div>

          {activeSection === "overview" && <OverviewSection />}
          {activeSection === "journal" && <JournalSection />}
          {activeSection === "sleep" && <SleepSection />}
          {activeSection === "tasks" && <TaskSection />}
          {activeSection === "reports" && <ReportSection />}
        </main>
      </div>
    </div>
  )
}
