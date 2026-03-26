"use client"

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { BookOpen, Moon, CheckSquare, TrendingUp } from "lucide-react"
import useSWR from "swr"

const fetcher = (url: string) => fetch(url).then(res => res.json())

interface Entry {
  id: string
  content: string
  mood?: string
  date: string
  timestamp: string
}

interface SleepEntry {
  duration_hours: number
  date: string
}

interface TasksResponse {
  tasks: Array<{ status: string }>
  exec_index: number
}

function fmtHrMin(hours: number) {
  const h = Math.floor(hours)
  const m = Math.round((hours - h) * 60)
  return `${h}h ${m}m`
}

export function OverviewSection() {
  const { data: entries = [] } = useSWR<Entry[]>("/api/journal?days=7", fetcher)
  const { data: sleepData = [] } = useSWR<(SleepEntry | null)[]>("/api/sleep?days=7", fetcher)
  const { data: tasksData } = useSWR<TasksResponse>("/api/tasks", fetcher)

  const todayEntries = entries.filter(e => {
    const d = new Date()
    const today = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`
    return e.date === today
  })

  const validSleep = sleepData.filter((s): s is SleepEntry => s !== null)
  const avgSleep = validSleep.length > 0
    ? validSleep.reduce((s, d) => s + d.duration_hours, 0) / validSleep.length
    : null

  const tasks = tasksData?.tasks || []
  const doneCount = tasks.filter(t => t.status === "done").length
  const execIndex = tasksData?.exec_index || 0

  const stats = [
    {
      title: "今日记录",
      value: todayEntries.length.toString(),
      subtitle: "条想法",
      icon: BookOpen,
      color: "text-chart-1",
    },
    {
      title: "平均睡眠",
      value: avgSleep !== null ? fmtHrMin(avgSleep) : "--",
      subtitle: "近7天",
      icon: Moon,
      color: "text-chart-2",
    },
    {
      title: "任务完成",
      value: tasks.length > 0 ? `${doneCount}/${tasks.length}` : "--",
      subtitle: "今日任务",
      icon: CheckSquare,
      color: "text-chart-3",
    },
    {
      title: "执行指数",
      value: tasks.length > 0 ? Math.round(execIndex).toString() : "--",
      subtitle: "效率评分",
      icon: TrendingUp,
      color: "text-chart-1",
    },
  ]

  return (
    <div className="space-y-6">
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {stats.map((stat) => {
          const Icon = stat.icon
          return (
            <Card key={stat.title} className="border-border bg-card">
              <CardHeader className="flex flex-row items-center justify-between pb-2">
                <CardTitle className="text-sm font-medium text-muted-foreground">
                  {stat.title}
                </CardTitle>
                <Icon className={`h-4 w-4 ${stat.color}`} />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold text-foreground">{stat.value}</div>
                <p className="text-xs text-muted-foreground">{stat.subtitle}</p>
              </CardContent>
            </Card>
          )
        })}
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        <Card className="border-border bg-card">
          <CardHeader>
            <CardTitle className="text-base font-medium">最近想法</CardTitle>
          </CardHeader>
          <CardContent>
            {entries.length === 0 ? (
              <p className="text-sm text-muted-foreground">暂无记录</p>
            ) : (
              <div className="space-y-3">
                {entries.slice(0, 5).map((entry) => (
                  <div key={entry.id} className="border-l-2 border-primary/30 pl-3">
                    <p className="line-clamp-2 text-sm text-foreground">{entry.content}</p>
                    <p className="mt-1 text-xs text-muted-foreground">
                      {entry.timestamp.split("T")[1]?.slice(0, 5)}
                      {entry.mood && <span className="ml-2">{entry.mood}</span>}
                    </p>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        <Card className="border-border bg-card">
          <CardHeader>
            <CardTitle className="text-base font-medium">睡眠趋势</CardTitle>
          </CardHeader>
          <CardContent>
            {validSleep.length === 0 ? (
              <p className="text-sm text-muted-foreground">暂无睡眠数据</p>
            ) : (
              <div className="flex h-[120px] items-end gap-2">
                {validSleep.slice(-7).map((sleep, i) => {
                  const maxH = 10
                  const pct = Math.min((sleep.duration_hours / maxH) * 100, 100)
                  return (
                    <div key={i} className="flex flex-1 flex-col items-center gap-1">
                      <div
                        className="w-full rounded-t bg-chart-2/80"
                        style={{ height: `${pct}%` }}
                        title={fmtHrMin(sleep.duration_hours)}
                      />
                      <span className="text-[10px] text-muted-foreground">
                        {sleep.date.slice(5).replace("-", "/")}
                      </span>
                    </div>
                  )
                })}
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
