"use client"

import { useState, useCallback, useMemo, useEffect, useRef } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { cn } from "@/lib/utils"
import { Moon, Clock } from "lucide-react"
import useSWR from "swr"

const API = "/api/sleep"

interface SleepEntry {
  id: string
  date: string
  bedtime: string
  wake_time: string
  duration_hours: number
  quality_score?: number
}

interface Popover {
  content: string
  x: number
  y: number
}

const fetcher = (url: string) => fetch(url).then(res => res.json())

function getLocalDateStr(date?: Date) {
  date = date || new Date()
  const y = date.getFullYear()
  const m = String(date.getMonth() + 1).padStart(2, "0")
  const d = String(date.getDate()).padStart(2, "0")
  return `${y}-${m}-${d}`
}

function fmtHrMin(hours: number) {
  const h = Math.floor(hours)
  const m = Math.round((hours - h) * 60)
  return `${h} hr ${m} min`
}

function toHours(timeStr: string) {
  if (!timeStr) return null
  const [h, m] = timeStr.split(":").map(Number)
  return h + m / 60
}

function normBedtime(h: number | null) {
  if (h === null) return null
  return h < 12 ? h + 24 : h
}

function TimeSelect24({ value, onChange, label }: { value: string; onChange: (v: string) => void; label: string }) {
  const [h, m] = value.split(":").map(Number)

  return (
    <div className="space-y-2">
      <label className="text-xs uppercase tracking-wide text-muted-foreground">{label}</label>
      <div className="flex items-center gap-1">
        <select
          value={h}
          onChange={(e) => onChange(`${String(e.target.value).padStart(2, "0")}:${String(m).padStart(2, "0")}`)}
          className="h-12 w-16 rounded-md border border-border bg-secondary/50 text-center text-xl font-light text-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
        >
          {Array.from({ length: 24 }, (_, i) => (
            <option key={i} value={i}>{i}</option>
          ))}
        </select>
        <span className="text-xl font-light text-muted-foreground">:</span>
        <select
          value={m}
          onChange={(e) => onChange(`${String(h).padStart(2, "0")}:${String(e.target.value).padStart(2, "0")}`)}
          className="h-12 w-16 rounded-md border border-border bg-secondary/50 text-center text-xl font-light text-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
        >
          {Array.from({ length: 60 }, (_, i) => (
            <option key={i} value={i}>{String(i).padStart(2, "0")}</option>
          ))}
        </select>
      </div>
    </div>
  )
}

export function SleepSection() {
  const { data: sleepData = [], mutate, isLoading } = useSWR<(SleepEntry | null)[]>(`${API}?days=7`, fetcher)
  const [date, setDate] = useState(getLocalDateStr())
  const [bedtime, setBedtime] = useState("23:00")
  const [waketime, setWaketime] = useState("07:00")
  const [quality, setQuality] = useState(0)
  const [error, setError] = useState("")
  const [saved, setSaved] = useState("")
  const [isSubmitting, setIsSubmitting] = useState(false)

  // Bar animation: trigger after data loads
  const [barsVisible, setBarsVisible] = useState(false)
  useEffect(() => {
    setBarsVisible(false)
    if (!isLoading) {
      const id = requestAnimationFrame(() => requestAnimationFrame(() => setBarsVisible(true)))
      return () => cancelAnimationFrame(id)
    }
  }, [isLoading, sleepData])

  // Popover state
  const [popover, setPopover] = useState<Popover | null>(null)
  const activeBarEl = useRef<HTMLElement | null>(null)

  useEffect(() => {
    const close = () => { setPopover(null); activeBarEl.current = null }
    document.addEventListener("click", close)
    return () => document.removeEventListener("click", close)
  }, [])

  const today = getLocalDateStr()

  const showError = useCallback((msg: string) => {
    setError(msg)
    setTimeout(() => setError(""), 3000)
  }, [])

  const handleSubmit = async () => {
    if (!bedtime || !waketime) {
      showError("请填写入睡和起床时间")
      return
    }
    setIsSubmitting(true)

    try {
      const res = await fetch(API, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          bedtime,
          wake_time: waketime,
          date: date || undefined,
          quality_score: quality || undefined,
        }),
      })

      if (res.status === 400) {
        const err = await res.json().catch(() => ({}))
        showError(err.error || "时间格式无效")
        return
      }
      if (!res.ok) {
        const body = await res.json().catch(() => ({}))
        throw new Error(body.error || "提交失败")
      }

      setSaved("已记录 ✓")
      setTimeout(() => setSaved(""), 2000)
      setQuality(0)
      mutate()
    } catch (err) {
      showError(err instanceof Error ? err.message : "记录失败")
    } finally {
      setIsSubmitting(false)
    }
  }

  const validData = useMemo(() => sleepData.filter((d): d is SleepEntry => d !== null), [sleepData])

  const avgHours = useMemo(() => {
    if (validData.length === 0) return null
    return validData.reduce((s, d) => s + d.duration_hours, 0) / validData.length
  }, [validData])

  const chartData = useMemo(() => {
    const dataMap: Record<string, SleepEntry> = {}
    validData.forEach(entry => { dataMap[entry.date] = entry })

    const now = new Date()
    const sunday = new Date(now)
    sunday.setDate(now.getDate() - now.getDay())

    return Array.from({ length: 7 }, (_, i) => {
      const d = new Date(sunday)
      d.setDate(sunday.getDate() + i)
      const dateStr = getLocalDateStr(d)
      const entry = dataMap[dateStr] || null
      const [, mo, day] = dateStr.split("-")
      return {
        dateStr,
        label: `${mo}/${day}`,
        isToday: dateStr === today,
        entry,
      }
    })
  }, [validData, today])

  const { yMin, yMax } = useMemo(() => {
    const bedHours = validData.map(d => normBedtime(toHours(d.bedtime))).filter((h): h is number => h !== null)
    const wakeHours = validData.map(d => toHours(d.wake_time)).filter((h): h is number => h !== null)

    if (bedHours.length > 0 && wakeHours.length > 0) {
      return {
        yMax: Math.max(...bedHours) + 1,
        yMin: Math.min(...wakeHours) - 1,
      }
    }
    return { yMax: 25, yMin: 6 }
  }, [validData])

  const range = yMax - yMin

  const handleBarClick = (e: React.MouseEvent<HTMLDivElement>, entry: SleepEntry) => {
    e.stopPropagation()
    const el = e.currentTarget

    // Toggle off if same bar clicked
    if (activeBarEl.current === el && popover) {
      setPopover(null)
      activeBarEl.current = null
      return
    }
    activeBarEl.current = el

    const rect = el.getBoundingClientRect()
    const content = `${fmtHrMin(entry.duration_hours)}\n${entry.bedtime} → ${entry.wake_time}`
    const popoverW = 140
    const popoverH = 52

    let left = rect.right + 10
    if (left + popoverW > window.innerWidth - 8) left = rect.left - popoverW - 10
    left = Math.max(8, left)
    let top = rect.top + rect.height / 2 - popoverH / 2
    top = Math.max(8, Math.min(top, window.innerHeight - popoverH - 8))

    setPopover({ content, x: left, y: top })
  }

  return (
    <div className="space-y-6">
      <Card className="border-border bg-card">
        <CardHeader className="pb-3">
          <CardTitle className="section-card-title text-base font-medium">
            <Moon className="h-4 w-4 text-primary" />
            记录睡眠
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex flex-wrap items-end gap-6">
            <div className="flex flex-col gap-2">
              <label className="text-xs uppercase tracking-wide text-muted-foreground">日期</label>
              <Input
                type="date"
                value={date}
                onChange={(e) => setDate(e.target.value)}
                max={today}
                className="h-12 w-auto border-border bg-secondary/50"
              />
            </div>
            <TimeSelect24 value={bedtime} onChange={setBedtime} label="入睡时间" />
            <TimeSelect24 value={waketime} onChange={setWaketime} label="起床时间" />
          </div>

          <div className="flex items-center gap-4">
            <label className="text-xs uppercase tracking-wide text-muted-foreground">睡眠质量</label>
            <div className="flex gap-1.5">
              {[1, 2, 3, 4, 5].map((v) => (
                <button
                  key={v}
                  type="button"
                  onClick={() => setQuality(quality === v ? 0 : v)}
                  className={cn(
                    "flex h-8 w-8 items-center justify-center rounded-lg border text-sm font-medium transition-colors",
                    v <= quality
                      ? "border-primary bg-primary/20 text-primary shadow-[0_0_12px_rgba(59,130,246,0.35)]"
                      : "border-border bg-secondary/50 text-muted-foreground hover:border-primary/50"
                  )}
                >
                  {v}
                </button>
              ))}
            </div>
          </div>

          <div className="flex items-center justify-between pt-2">
            <div className="flex items-center gap-2">
              {error && <span className="text-sm text-destructive">{error}</span>}
              {saved && <span className="text-sm text-green-400 font-medium">{saved}</span>}
            </div>
            <Button
              onClick={handleSubmit}
              disabled={isSubmitting}
              className="bg-primary text-primary-foreground hover:bg-primary/90 shadow-[0_0_18px_rgba(59,130,246,0.35)]"
            >
              记录睡眠
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card className="border-border bg-card">
        <CardHeader className="pb-2">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-xs uppercase tracking-wider text-muted-foreground">Sleep Duration</p>
              <p className="text-2xl font-bold tabular-nums text-foreground">
                {avgHours !== null ? `${fmtHrMin(avgHours)} avg` : "— hr — min"}
              </p>
            </div>
            <Clock className="h-5 w-5 text-muted-foreground" />
          </div>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex h-[220px] items-center justify-center">
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
            <div className="flex gap-2">
              {/* Y-axis */}
              <div className="relative h-[220px] w-10 shrink-0">
                {Array.from({ length: Math.floor((yMax - yMin) / 2) + 1 }, (_, i) => {
                  const h = Math.ceil(yMin / 2) * 2 + i * 2
                  if (h > yMax) return null
                  const pct = ((yMax - h) / range) * 100
                  const displayH = h % 24
                  return (
                    <span
                      key={h}
                      className="absolute right-0 -translate-y-1/2 text-[10px] tabular-nums text-muted-foreground"
                      style={{ top: `${pct}%` }}
                    >
                      {String(displayH).padStart(2, "0")}:00
                    </span>
                  )
                })}
              </div>

              {/* Bars area */}
              <div className="relative flex flex-1 gap-1">
                {/* Grid lines */}
                <div className="pointer-events-none absolute inset-0">
                  {Array.from({ length: Math.floor((yMax - yMin) / 2) + 1 }, (_, i) => {
                    const h = Math.ceil(yMin / 2) * 2 + i * 2
                    if (h > yMax) return null
                    const pct = ((yMax - h) / range) * 100
                    return (
                      <div
                        key={h}
                        className="absolute left-0 right-0 border-t border-border/40"
                        style={{ top: `${pct}%` }}
                      />
                    )
                  })}
                </div>

                {chartData.map((day) => {
                  let topPct = 0
                  let heightPct = 0
                  if (day.entry) {
                    const bedH = normBedtime(toHours(day.entry.bedtime))!
                    const wakeH = toHours(day.entry.wake_time)!
                    topPct = ((yMax - bedH) / range) * 100
                    heightPct = ((bedH - wakeH) / range) * 100
                  }

                  return (
                    <div key={day.dateStr} className="flex flex-1 flex-col items-center">
                      <div className="relative h-[220px] w-full overflow-hidden">
                        {day.entry && (
                          <div
                            className="absolute left-[18%] right-[18%] cursor-pointer rounded"
                            style={{
                              top: `${topPct}%`,
                              height: barsVisible ? `${heightPct}%` : "0%",
                              minHeight: barsVisible ? 4 : 0,
                              transition: "height 0.4s ease-out, box-shadow 0.2s",
                              background: "linear-gradient(180deg, #60A5FA 0%, #3B82F6 60%, #2563EB 100%)",
                              boxShadow: "0 0 12px rgba(59, 130, 246, 0.45)",
                            }}
                            onClick={(e) => handleBarClick(e, day.entry!)}
                          />
                        )}
                      </div>
                      <span className={cn(
                        "mt-1 text-[10px] tabular-nums",
                        day.isToday ? "font-bold text-primary" : "text-muted-foreground"
                      )}>
                        {day.label}
                      </span>
                    </div>
                  )
                })}
              </div>
            </div>
          )}

          {!isLoading && !chartData.find(d => d.dateStr === today && d.entry) && (
            <p className="mt-3 text-center text-xs text-muted-foreground">
              今晚记得记录睡眠
            </p>
          )}
        </CardContent>
      </Card>

      {/* Sleep detail popover */}
      {popover && (
        <div
          className="fixed z-50 whitespace-pre rounded-xl border border-primary/30 bg-popover px-3.5 py-2.5 text-sm text-popover-foreground tabular-nums shadow-[0_8px_40px_rgba(0,0,0,0.7),0_0_24px_rgba(59,130,246,0.12)]"
          style={{ left: popover.x, top: popover.y, minWidth: 120 }}
        >
          {popover.content}
        </div>
      )}
    </div>
  )
}
