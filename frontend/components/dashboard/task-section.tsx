"use client"

import { useState, useCallback, useMemo } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { cn } from "@/lib/utils"
import { CheckSquare, Check, Minus, X, Plus } from "lucide-react"
import useSWR from "swr"

const API = "/api/tasks"

interface Task {
  id: string
  title: string
  quadrant: "q1" | "q2" | "q3" | "q4"
  status: "todo" | "done" | "skipped"
  skip_reason?: string
}

interface TasksResponse {
  tasks: Task[]
  exec_index: number
}

const quadrants = [
  { id: "q1" as const, label: "Q1 紧急重要", color: "bg-[var(--q1)]" },
  { id: "q2" as const, label: "Q2 重要", color: "bg-[var(--q2)]" },
  { id: "q3" as const, label: "Q3 紧急", color: "bg-[var(--q3)]" },
  { id: "q4" as const, label: "Q4 其他", color: "bg-[var(--q4)]" },
]

const fetcher = (url: string) => fetch(url).then(res => res.json())

// McLaren-style speedometer gauge
function ExecutionGauge({ value, done, total }: { value: number; done: number; total: number }) {
  const clampedValue = Math.min(Math.max(value, 0), 100)
  const angle = -135 + (clampedValue / 100) * 270 // -135 to 135 degrees
  
  // Generate tick marks
  const ticks = Array.from({ length: 11 }, (_, i) => {
    const tickAngle = -135 + (i / 10) * 270
    const radian = (tickAngle * Math.PI) / 180
    const innerRadius = i % 2 === 0 ? 72 : 76
    const outerRadius = 82
    const x1 = 100 + innerRadius * Math.cos(radian)
    const y1 = 100 + innerRadius * Math.sin(radian)
    const x2 = 100 + outerRadius * Math.cos(radian)
    const y2 = 100 + outerRadius * Math.sin(radian)
    return { x1, y1, x2, y2, major: i % 2 === 0, label: i * 10, angle: tickAngle }
  })
  
  // Arc path for the gauge track
  const createArc = (startAngle: number, endAngle: number, radius: number) => {
    const startRad = (startAngle * Math.PI) / 180
    const endRad = (endAngle * Math.PI) / 180
    const x1 = 100 + radius * Math.cos(startRad)
    const y1 = 100 + radius * Math.sin(startRad)
    const x2 = 100 + radius * Math.cos(endRad)
    const y2 = 100 + radius * Math.sin(endRad)
    const largeArc = endAngle - startAngle > 180 ? 1 : 0
    return `M ${x1} ${y1} A ${radius} ${radius} 0 ${largeArc} 1 ${x2} ${y2}`
  }
  
  // Needle rotation
  const needleAngle = angle
  
  // Color zones
  const getZoneColor = (val: number) => {
    if (val < 30) return "#ef4444" // red
    if (val < 60) return "#f97316" // orange
    if (val < 80) return "#eab308" // yellow
    return "#22c55e" // green
  }

  return (
    <div className="relative flex flex-col items-center">
      <svg viewBox="0 0 200 140" className="w-full max-w-[320px]">
        <defs>
          {/* Gradient for the arc */}
          <linearGradient id="gaugeGradient" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="#ef4444" />
            <stop offset="30%" stopColor="#f97316" />
            <stop offset="60%" stopColor="#eab308" />
            <stop offset="100%" stopColor="#22c55e" />
          </linearGradient>
          
          {/* Glow filter */}
          <filter id="glow" x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur stdDeviation="2" result="coloredBlur" />
            <feMerge>
              <feMergeNode in="coloredBlur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
          
          {/* Needle shadow */}
          <filter id="needleShadow" x="-50%" y="-50%" width="200%" height="200%">
            <feDropShadow dx="0" dy="1" stdDeviation="2" floodOpacity="0.5" />
          </filter>
        </defs>
        
        {/* Background arc track */}
        <path
          d={createArc(-135, 135, 85)}
          fill="none"
          style={{ stroke: "var(--color-secondary)" }}
          strokeWidth="8"
          strokeLinecap="round"
        />
        
        {/* Colored progress arc */}
        <path
          d={createArc(-135, -135 + (clampedValue / 100) * 270, 85)}
          fill="none"
          stroke="url(#gaugeGradient)"
          strokeWidth="8"
          strokeLinecap="round"
          filter="url(#glow)"
          className="transition-all duration-700 ease-out"
        />
        
        {/* Tick marks */}
        {ticks.map((tick, i) => (
          <g key={i}>
            <line
              x1={tick.x1}
              y1={tick.y1}
              x2={tick.x2}
              y2={tick.y2}
              style={{ stroke: tick.major ? "var(--color-foreground)" : "var(--color-muted-foreground)" }}
              strokeWidth={tick.major ? 2 : 1}
              opacity={tick.major ? 0.8 : 0.4}
            />
            {tick.major && (
              <text
                x={100 + 58 * Math.cos((tick.angle * Math.PI) / 180)}
                y={100 + 58 * Math.sin((tick.angle * Math.PI) / 180)}
                textAnchor="middle"
                dominantBaseline="middle"
                style={{ fill: "var(--color-muted-foreground)", fontSize: "8px", fontWeight: 500 }}
              >
                {tick.label}
              </text>
            )}
          </g>
        ))}
        
        {/* Center hub */}
        <circle cx="100" cy="100" r="18" style={{ fill: "var(--color-card)", stroke: "var(--color-border)" }} strokeWidth="2" />
        <circle cx="100" cy="100" r="12" style={{ fill: "var(--color-secondary)" }} />
        <circle cx="100" cy="100" r="6" fill={getZoneColor(clampedValue)} filter="url(#glow)" />
        
        {/* Needle */}
        <g
          transform={`rotate(${needleAngle}, 100, 100)`}
          filter="url(#needleShadow)"
          className="transition-transform duration-700 ease-out"
        >
          <polygon
            points="100,30 96,100 100,108 104,100"
            style={{ fill: "var(--color-foreground)" }}
          />
          <polygon
            points="100,35 98,95 100,100 102,95"
            fill={getZoneColor(clampedValue)}
          />
        </g>
        
        {/* Digital readout */}
        <rect x="70" y="115" width="60" height="20" rx="4" style={{ fill: "var(--color-secondary)", stroke: "var(--color-border)" }} strokeWidth="1" />
        <text
          x="100"
          y="128"
          textAnchor="middle"
          dominantBaseline="middle"
          style={{ fill: "var(--color-foreground)", fontSize: "12px", fontWeight: 700, fontFamily: "monospace" }}
        >
          {total > 0 ? Math.round(clampedValue) : "---"}
        </text>
      </svg>
      
      {/* Labels */}
      <div className="mt-2 flex flex-col items-center gap-1">
        <span className="text-xs uppercase tracking-widest text-muted-foreground">Execution Index</span>
        <span className="text-sm text-muted-foreground">
          {total > 0 ? `${done}/${total} 完成` : "今日任务"}
        </span>
      </div>
    </div>
  )
}

export function TaskSection() {
  const { data, mutate, isLoading } = useSWR<TasksResponse>(API, fetcher)
  const tasks = data?.tasks || []
  const execIndex = data?.exec_index || 0
  
  const [title, setTitle] = useState("")
  const [selectedQuad, setSelectedQuad] = useState<"q1" | "q2" | "q3" | "q4">("q2")
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [showAddForm, setShowAddForm] = useState(false)

  const tasksByQuad = useMemo(() => {
    const grouped: Record<string, Task[]> = { q1: [], q2: [], q3: [], q4: [] }
    tasks.forEach(t => {
      if (grouped[t.quadrant]) grouped[t.quadrant].push(t)
    })
    return grouped
  }, [tasks])

  const stats = useMemo(() => {
    const done = tasks.filter(t => t.status === "done").length
    const total = tasks.length
    return { done, total }
  }, [tasks])

  const handleAdd = async () => {
    if (!title.trim()) return
    setIsSubmitting(true)
    
    try {
      const res = await fetch(API, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ title: title.trim(), quadrant: selectedQuad }),
      })
      
      if (!res.ok) throw new Error("添加失败")
      
      setTitle("")
      mutate()
    } catch {
      // ignore
    } finally {
      setIsSubmitting(false)
    }
  }

  const patchTask = useCallback(async (id: string, patch: Partial<Task>) => {
    try {
      await fetch(`${API}/${encodeURIComponent(id)}`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(patch),
      })
      mutate()
    } catch {
      // ignore
    }
  }, [mutate])

  const deleteTask = useCallback(async (id: string) => {
    try {
      await fetch(`${API}/${encodeURIComponent(id)}`, { method: "DELETE" })
      mutate()
    } catch {
      // ignore
    }
  }, [mutate])

  return (
    <div className="space-y-6">
      {/* Task Matrix - Now at the top */}
      <Card className="border-border bg-card overflow-hidden">
        <CardHeader className="pb-2 flex flex-row items-center justify-between">
          <CardTitle className="text-base font-medium">任务矩阵</CardTitle>
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8 rounded-full hover:bg-primary/10 hover:text-primary"
            onClick={() => setShowAddForm(!showAddForm)}
          >
            <Plus className={cn("h-5 w-5 transition-transform", showAddForm && "rotate-45")} />
          </Button>
        </CardHeader>
        
        {/* Inline Add Form */}
        {showAddForm && (
          <div className="border-b border-border px-4 pb-4 space-y-3">
            <Input
              placeholder="输入任务名称..."
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              className="border-border bg-secondary/50"
              autoFocus
              onKeyDown={(e) => {
                if (e.key === "Enter" && title.trim()) {
                  e.preventDefault()
                  handleAdd()
                  setShowAddForm(false)
                }
                if (e.key === "Escape") {
                  setShowAddForm(false)
                  setTitle("")
                }
              }}
            />
            
            <div className="flex flex-wrap gap-2">
              {quadrants.map((q) => (
                <button
                  key={q.id}
                  onClick={() => setSelectedQuad(q.id)}
                  className={cn(
                    "flex-1 rounded-full border px-3 py-1.5 text-center text-xs font-medium transition-colors",
                    selectedQuad === q.id
                      ? `${q.color} border-transparent text-white`
                      : "border-border bg-secondary/50 text-muted-foreground hover:text-foreground"
                  )}
                >
                  {q.label}
                </button>
              ))}
            </div>
            
            <div className="flex justify-end gap-2">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => {
                  setShowAddForm(false)
                  setTitle("")
                }}
              >
                取消
              </Button>
              <Button
                size="sm"
                onClick={() => {
                  handleAdd()
                  setShowAddForm(false)
                }}
                disabled={!title.trim() || isSubmitting}
                className="bg-primary text-primary-foreground hover:bg-primary/90"
              >
                添加
              </Button>
            </div>
          </div>
        )}
        
        <CardContent className="p-0">
          {isLoading ? (
            <div className="flex justify-center py-12">
              <div className="flex gap-1.5">
                {[0, 1, 2].map((i) => (
                  <span
                    key={i}
                    className="h-2 w-2 animate-pulse rounded-full bg-muted-foreground"
                    style={{ animationDelay: `${i * 0.2}s` }}
                  />
                ))}
              </div>
            </div>
          ) : (
            <div className="grid grid-cols-2">
              {quadrants.map((q, idx) => (
                <div
                  key={q.id}
                  className={cn(
                    "min-h-[120px] p-4",
                    idx % 2 === 0 && "border-r border-border",
                    idx < 2 && "border-b border-border"
                  )}
                >
                  <div className="mb-3 flex items-center gap-2">
                    <span className={cn("h-2 w-2 rounded-full", q.color)} />
                    <span className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                      {q.label}
                    </span>
                  </div>
                  
                  <div className="space-y-1">
                    {tasksByQuad[q.id].map((task) => (
                      <TaskRow
                        key={task.id}
                        task={task}
                        onToggle={() => patchTask(task.id, { status: task.status === "done" ? "todo" : "done" })}
                        onSkip={() => patchTask(task.id, { status: "skipped" })}
                        onDelete={() => deleteTask(task.id)}
                      />
                    ))}
                    {tasksByQuad[q.id].length === 0 && (
                      <p className="text-xs text-muted-foreground/60">暂无任务</p>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
      
    </div>
  )
}

function TaskRow({
  task,
  onToggle,
  onSkip,
  onDelete,
}: {
  task: Task
  onToggle: () => void
  onSkip: () => void
  onDelete: () => void
}) {
  const isDone = task.status === "done"
  const isSkipped = task.status === "skipped"
  
  return (
    <div className={cn(
      "group flex items-center gap-2 rounded-lg p-1.5 transition-colors hover:bg-secondary/50",
      (isDone || isSkipped) && "opacity-60"
    )}>
      <button
        onClick={onToggle}
        className={cn(
          "flex h-5 w-5 shrink-0 items-center justify-center rounded-full border transition-colors",
          isDone
            ? "border-primary bg-primary text-primary-foreground"
            : isSkipped
            ? "border-muted bg-muted text-muted-foreground"
            : "border-border hover:border-primary"
        )}
      >
        {isDone && <Check className="h-3 w-3" />}
        {isSkipped && <Minus className="h-3 w-3" />}
      </button>
      
      <span className={cn(
        "flex-1 text-sm text-foreground",
        isDone && "line-through text-muted-foreground",
        isSkipped && "text-muted-foreground"
      )}>
        {task.title}
      </span>
      
      <div className="flex gap-1 opacity-0 transition-opacity group-hover:opacity-100">
        {!isSkipped && (
          <button
            onClick={onSkip}
            className="rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
            title="跳过"
          >
            <Minus className="h-3.5 w-3.5" />
          </button>
        )}
        <button
          onClick={onDelete}
          className="rounded p-1 text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
          title="删除"
        >
          <X className="h-3.5 w-3.5" />
        </button>
      </div>
    </div>
  )
}
