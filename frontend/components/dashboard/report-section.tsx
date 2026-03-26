"use client"

import { useState, useRef, useCallback } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { cn } from "@/lib/utils"
import { BarChart2, Plus, Trash2, ChevronRight, Loader2 } from "lucide-react"
import useSWR from "swr"

interface ReportListItem {
  id: string
  generated_at: string
  type: string
  date: string
  contradiction_score: number
  intensity_level: number
}

interface Report {
  id: string
  generated_at: string
  type: string
  date: string
  period_start: string
  period_end: string
  contradiction_score: number
  intensity_level: number
  analysis: string
}

const fetcher = (url: string) => fetch(url).then(res => res.json())

function getLocalDateStr() {
  const d = new Date()
  const y = d.getFullYear()
  const m = String(d.getMonth() + 1).padStart(2, "0")
  const day = String(d.getDate()).padStart(2, "0")
  return `${y}-${m}-${day}`
}

function reportLabel(r: { type: string; date: string }) {
  return `${r.date} ${r.type === "weekly" ? "周报" : "日报"}`
}

export function ReportSection() {
  const { data: rawReports, mutate, isLoading } = useSWR<ReportListItem[]>("/api/reports", fetcher)
  const reports = Array.isArray(rawReports) ? rawReports : []

  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [showForm, setShowForm] = useState(false)
  const [date, setDate] = useState(getLocalDateStr())
  const [reportType, setReportType] = useState<"daily" | "weekly">("daily")
  const [isGenerating, setIsGenerating] = useState(false)
  const [streamContent, setStreamContent] = useState("")
  const [streamMeta, setStreamMeta] = useState<{ date: string; type: string } | null>(null)
  const [error, setError] = useState("")
  const streamRef = useRef<string>("")

  const { data: selectedReport } = useSWR<Report>(
    selectedId ? `/api/reports/${encodeURIComponent(selectedId)}` : null,
    fetcher
  )

  const handleDelete = useCallback(async (id: string, e: React.MouseEvent) => {
    e.stopPropagation()
    try {
      await fetch(`/api/reports/${encodeURIComponent(id)}`, { method: "DELETE" })
      if (selectedId === id) setSelectedId(null)
      mutate()
    } catch {
      // ignore
    }
  }, [selectedId, mutate])

  const handleGenerate = async () => {
    setIsGenerating(true)
    setStreamContent("")
    setStreamMeta(null)
    streamRef.current = ""
    setError("")
    setSelectedId(null)

    try {
      const res = await fetch("/api/reports/generate", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        // Rust expects "type" (serde rename), not "report_type"
        body: JSON.stringify({ date, type: reportType }),
      })

      if (!res.ok || !res.body) {
        setError("生成失败，请检查后端服务")
        setIsGenerating(false)
        return
      }

      const reader = res.body.getReader()
      const decoder = new TextDecoder()
      let buffer = ""

      while (true) {
        const { done, value } = await reader.read()
        if (done) break

        buffer += decoder.decode(value, { stream: true })
        const lines = buffer.split("\n")
        buffer = lines.pop() ?? ""

        for (const line of lines) {
          if (!line.startsWith("data: ")) continue
          const raw = line.slice(6).trim()
          if (!raw) continue
          try {
            const event = JSON.parse(raw)
            if (event.type === "summary") {
              setStreamMeta({ date: event.date, type: event.report_type })
            } else if (event.type === "token") {
              // Rust sends "content", not "token"
              streamRef.current += event.content
              setStreamContent(streamRef.current)
            } else if (event.type === "done") {
              mutate()
              setShowForm(false)
              if (event.report_id) setSelectedId(event.report_id)
            } else if (event.type === "error") {
              // Rust sends "message", not "error"
              setError(event.message || "生成失败")
            }
          } catch {
            // non-JSON line, ignore
          }
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "生成失败")
    } finally {
      setIsGenerating(false)
    }
  }

  const displayContent = selectedReport?.analysis ?? (isGenerating || streamContent ? streamContent : null)
  const displayLabel = selectedReport
    ? reportLabel(selectedReport)
    : streamMeta
    ? reportLabel(streamMeta)
    : null

  return (
    <div className="grid gap-6 lg:grid-cols-[280px_1fr]">
      {/* History panel */}
      <div className="space-y-4">
        <Card className="border-border bg-card">
          <CardHeader className="flex flex-row items-center justify-between pb-3">
            <CardTitle className="section-card-title text-base font-medium">
              <BarChart2 className="h-4 w-4 text-primary" />
              历史报告
            </CardTitle>
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7 rounded-full hover:bg-primary/10 hover:text-primary"
              onClick={() => {
                setShowForm(!showForm)
                setSelectedId(null)
                setStreamContent("")
                setStreamMeta(null)
              }}
              title="生成新报告"
            >
              <Plus className={cn("h-4 w-4 transition-transform", showForm && "rotate-45")} />
            </Button>
          </CardHeader>
          <CardContent className="p-0">
            {isLoading ? (
              <div className="flex justify-center py-8">
                <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
              </div>
            ) : reports.length === 0 && !showForm ? (
              <p className="px-4 pb-4 text-xs text-muted-foreground">暂无报告，点击 + 生成</p>
            ) : (
              <ul className="divide-y divide-border">
                {reports.map((r) => (
                  <li
                    key={r.id}
                    onClick={() => { setSelectedId(r.id); setShowForm(false); setStreamContent("") }}
                    className={cn(
                      "group flex cursor-pointer items-center gap-2 px-4 py-3 transition-colors hover:bg-secondary/50",
                      selectedId === r.id && "bg-secondary/70"
                    )}
                  >
                    <div className="min-w-0 flex-1">
                      <p className="truncate text-sm font-medium text-foreground">{reportLabel(r)}</p>
                      <p className="text-xs text-muted-foreground">
                        矛盾分 {r.contradiction_score.toFixed(1)} · 强度 {r.intensity_level}
                      </p>
                    </div>
                    <div className="flex shrink-0 items-center gap-1 opacity-0 transition-opacity group-hover:opacity-100">
                      <button
                        onClick={(e) => handleDelete(r.id, e)}
                        className="rounded p-1 text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                        title="删除"
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                      </button>
                      <ChevronRight className="h-3.5 w-3.5 text-muted-foreground" />
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Main content area */}
      <div>
        {showForm ? (
          <Card className="border-border bg-card">
            <CardHeader className="pb-3">
              <CardTitle className="text-base font-medium">生成报告</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex flex-wrap gap-4">
                <div className="flex flex-col gap-1.5">
                  <label className="text-xs uppercase tracking-wide text-muted-foreground">日期</label>
                  <Input
                    type="date"
                    value={date}
                    onChange={(e) => setDate(e.target.value)}
                    max={getLocalDateStr()}
                    className="h-10 w-auto border-border bg-secondary/50"
                  />
                </div>
                <div className="flex flex-col gap-1.5">
                  <label className="text-xs uppercase tracking-wide text-muted-foreground">类型</label>
                  <div className="flex gap-2">
                    {(["daily", "weekly"] as const).map((t) => (
                      <button
                        key={t}
                        onClick={() => setReportType(t)}
                        className={cn(
                          "rounded-full border px-4 py-1.5 text-xs font-medium transition-colors",
                          reportType === t
                            ? "border-primary bg-primary/20 text-primary"
                            : "border-border bg-secondary/50 text-muted-foreground hover:border-primary/50"
                        )}
                      >
                        {t === "daily" ? "日报" : "周报"}
                      </button>
                    ))}
                  </div>
                </div>
              </div>

              {error && <p className="text-sm text-destructive">{error}</p>}

              {(isGenerating || streamContent) && (
                <div className="rounded-lg border border-border bg-secondary/30 p-4">
                  {displayLabel && (
                    <p className="mb-2 text-sm font-semibold text-foreground">{displayLabel}</p>
                  )}
                  <pre className="whitespace-pre-wrap font-sans text-sm text-foreground/90 leading-relaxed">
                    {streamContent}
                    {isGenerating && <span className="animate-pulse text-primary">▋</span>}
                  </pre>
                </div>
              )}

              <div className="flex justify-end gap-2 pt-2">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => { setShowForm(false); setStreamContent(""); setStreamMeta(null) }}
                  disabled={isGenerating}
                >
                  取消
                </Button>
                <Button
                  size="sm"
                  onClick={handleGenerate}
                  disabled={isGenerating || !date}
                  className="bg-primary text-primary-foreground hover:bg-primary/90 shadow-[0_0_18px_rgba(59,130,246,0.35)]"
                >
                  {isGenerating ? (
                    <>
                      <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                      生成中...
                    </>
                  ) : (
                    "生成报告"
                  )}
                </Button>
              </div>
            </CardContent>
          </Card>
        ) : displayContent !== null ? (
          <Card className="border-border bg-card">
            <CardHeader className="pb-3">
              <div className="flex items-start justify-between gap-4">
                <div>
                  <CardTitle className="text-base font-medium">
                    {displayLabel ?? "分析报告"}
                  </CardTitle>
                  {selectedReport && (
                    <p className="mt-1 text-xs text-muted-foreground">
                      {selectedReport.period_start} → {selectedReport.period_end}
                      {" · "}矛盾分 {selectedReport.contradiction_score.toFixed(1)}
                      {" · "}强度 {selectedReport.intensity_level}
                    </p>
                  )}
                </div>
                {selectedReport && (
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-7 w-7 shrink-0 rounded hover:bg-destructive/10 hover:text-destructive"
                    onClick={(e) => handleDelete(selectedReport.id, e)}
                    title="删除报告"
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                )}
              </div>
            </CardHeader>
            <CardContent>
              <div className="rounded-lg border border-border bg-secondary/30 p-4">
                <pre className="whitespace-pre-wrap font-sans text-sm text-foreground/90 leading-relaxed">
                  {displayContent}
                </pre>
              </div>
            </CardContent>
          </Card>
        ) : (
          <div className="flex h-64 items-center justify-center rounded-xl border border-dashed border-border">
            <div className="text-center">
              <BarChart2 className="mx-auto mb-3 h-10 w-10 text-muted-foreground/40" />
              <p className="text-sm text-muted-foreground">选择历史报告或生成新报告</p>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
