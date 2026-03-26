"use client"

import { useState, useCallback } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Textarea } from "@/components/ui/textarea"
import { Input } from "@/components/ui/input"
import { cn } from "@/lib/utils"
import { Trash2, PenLine } from "lucide-react"
import useSWR from "swr"

const API = "/api/journal"

interface Entry {
  id: string
  content: string
  mood?: string
  tags?: string[]
  date: string
  timestamp: string
}

const moods = ["😞", "😐", "🙂", "😊"]

const fetcher = (url: string) => fetch(url).then(res => res.json())

function timeStr(timestamp: string) {
  const parts = (timestamp || "").split("T")
  return parts.length >= 2 ? parts[1].slice(0, 5) : ""
}

function getLocalDateStr(date?: Date) {
  date = date || new Date()
  const y = date.getFullYear()
  const m = String(date.getMonth() + 1).padStart(2, "0")
  const d = String(date.getDate()).padStart(2, "0")
  return `${y}-${m}-${d}`
}

function dateLabel(dateStr: string) {
  const today = getLocalDateStr()
  const yesterday = getLocalDateStr(new Date(Date.now() - 86400000))
  if (dateStr === today) return "Today"
  if (dateStr === yesterday) return "Yesterday"
  const [y, mo, d] = dateStr.split("-").map(Number)
  return new Date(y, mo - 1, d).toLocaleDateString("en-US", { month: "long", day: "numeric" })
}

function groupByDate(entries: Entry[]) {
  const groups: Record<string, Entry[]> = {}
  for (const entry of entries) {
    if (!groups[entry.date]) groups[entry.date] = []
    groups[entry.date].push(entry)
  }
  return groups
}

export function JournalSection() {
  const { data: entries = [], mutate, isLoading } = useSWR<Entry[]>(`${API}?days=7`, fetcher)
  const [content, setContent] = useState("")
  const [selectedMood, setSelectedMood] = useState("")
  const [tags, setTags] = useState("")
  const [error, setError] = useState("")
  const [isSubmitting, setIsSubmitting] = useState(false)

  const showError = useCallback((msg: string) => {
    setError(msg)
    setTimeout(() => setError(""), 3000)
  }, [])

  const handleSubmit = async () => {
    if (!content.trim()) return
    setIsSubmitting(true)
    
    try {
      const payload = {
        content: content.trim(),
        mood: selectedMood || undefined,
        tags: tags.trim() ? tags.trim().split(/\s+/) : undefined,
      }
      
      const res = await fetch(API, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      })
      
      if (!res.ok) {
        const body = await res.json().catch(() => ({}))
        throw new Error(body.error || "提交失败")
      }
      
      const entry = await res.json()
      mutate([entry, ...entries], false)
      setContent("")
      setSelectedMood("")
      setTags("")
    } catch (err) {
      showError(err instanceof Error ? err.message : "提交失败")
    } finally {
      setIsSubmitting(false)
    }
  }

  const handleDelete = async (id: string) => {
    if (!confirm("删除这条记录？")) return
    
    try {
      const res = await fetch(`${API}/${encodeURIComponent(id)}`, { method: "DELETE" })
      if (!res.ok) {
        showError("删除失败，请重试")
        return
      }
      mutate(entries.filter(e => e.id !== id), false)
    } catch {
      showError("删除失败")
    }
  }

  const groups = groupByDate(entries)

  return (
    <div className="space-y-6">
      <Card className="border-border bg-card">
        <CardHeader className="pb-3">
          <CardTitle className="section-card-title text-base font-medium">
            <PenLine className="h-4 w-4 text-primary" />
            写点什么
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <Textarea
            placeholder="今天在想什么？"
            value={content}
            onChange={(e) => setContent(e.target.value)}
            className="min-h-[100px] resize-none border-border bg-secondary/50 text-foreground placeholder:text-muted-foreground"
            onKeyDown={(e) => {
              if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
                e.preventDefault()
                if (content.trim()) handleSubmit()
              }
            }}
          />
          
          <div className="flex flex-wrap items-center gap-4">
            <div className="flex items-center gap-2">
              <span className="text-xs uppercase tracking-wide text-muted-foreground">今天状态</span>
              <div className="flex gap-1">
                {moods.map((mood) => (
                  <button
                    key={mood}
                    type="button"
                    onClick={() => setSelectedMood(selectedMood === mood ? "" : mood)}
                    className={cn(
                      "flex h-9 w-9 items-center justify-center rounded-full border-2 text-xl transition-all hover:scale-110",
                      selectedMood === mood
                        ? "border-primary bg-primary/10"
                        : "border-transparent"
                    )}
                  >
                    {mood}
                  </button>
                ))}
              </div>
            </div>
            
            <div className="flex flex-1 items-center gap-2">
              <span className="text-xs uppercase tracking-wide text-muted-foreground">标签</span>
              <Input
                placeholder="空格分隔"
                value={tags}
                onChange={(e) => setTags(e.target.value)}
                className="h-8 max-w-[200px] border-border bg-secondary/50 text-sm"
              />
            </div>
          </div>
          
          <div className="flex items-center justify-between">
            {error && <span className="text-sm text-destructive">{error}</span>}
            <div className="ml-auto">
              <Button
                onClick={handleSubmit}
                disabled={!content.trim() || isSubmitting}
                className="bg-primary text-primary-foreground hover:bg-primary/90"
              >
                记录
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
      
      <div className="space-y-4">
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
        ) : entries.length === 0 ? (
          <Card className="border-border bg-card">
            <CardContent className="flex flex-col items-center justify-center py-12">
              <div className="mb-4 text-4xl">📝</div>
              <h3 className="text-lg font-semibold text-foreground">还没有记录</h3>
              <p className="text-sm text-muted-foreground">写下今天的第一个想法</p>
            </CardContent>
          </Card>
        ) : (
          Object.entries(groups).map(([date, dayEntries]) => (
            <div key={date} className="space-y-2">
              <div className="flex items-center gap-3">
                <span className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                  {dateLabel(date)}
                </span>
                <div className="h-px flex-1 bg-border" />
              </div>
              
              {dayEntries.map((entry) => (
                <Card key={entry.id} className="group border-border bg-card transition-colors hover:bg-card/80">
                  <CardContent className="relative py-4">
                    <div className="mb-2 text-xs text-muted-foreground">
                      {timeStr(entry.timestamp)}
                      {entry.mood && <span className="ml-2">{entry.mood}</span>}
                    </div>
                    <p className="whitespace-pre-wrap text-foreground">{entry.content}</p>
                    {entry.tags && entry.tags.length > 0 && (
                      <div className="mt-2 flex flex-wrap gap-1">
                        {entry.tags.map((tag, i) => (
                          <span key={i} className="rounded-full bg-secondary px-2 py-0.5 text-xs text-muted-foreground">
                            #{tag}
                          </span>
                        ))}
                      </div>
                    )}
                    <button
                      onClick={() => handleDelete(entry.id)}
                      className="absolute bottom-3 right-3 rounded-md p-1.5 text-muted-foreground opacity-0 transition-opacity hover:bg-destructive/10 hover:text-destructive group-hover:opacity-100"
                    >
                      <Trash2 className="h-4 w-4" />
                    </button>
                  </CardContent>
                </Card>
              ))}
            </div>
          ))
        )}
      </div>
    </div>
  )
}
