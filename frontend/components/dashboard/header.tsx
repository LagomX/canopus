"use client"

import { useEffect, useState } from "react"
import { useTheme } from "next-themes"
import { Moon, Sun } from "lucide-react"
import { Button } from "@/components/ui/button"

export function Header() {
  const [dateStr, setDateStr] = useState("")
  const { theme, setTheme } = useTheme()
  const [mounted, setMounted] = useState(false)
  
  useEffect(() => {
    setMounted(true)
    const d = new Date()
    const opts: Intl.DateTimeFormatOptions = { 
      weekday: "long", 
      month: "long", 
      day: "numeric" 
    }
    setDateStr(d.toLocaleDateString("zh-CN", opts))
  }, [])
  
  return (
    <header className="sticky top-0 z-30 flex h-16 items-center justify-between border-b border-border bg-background/95 px-6 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div>
        <h1 className="text-xl font-semibold text-foreground">仪表盘</h1>
        <p className="text-sm text-muted-foreground">{dateStr}</p>
      </div>
      
      {mounted && (
        <Button
          variant="ghost"
          size="icon"
          onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
          className="h-9 w-9 rounded-full"
        >
          {theme === "dark" ? (
            <Sun className="h-5 w-5 text-muted-foreground transition-colors hover:text-foreground" />
          ) : (
            <Moon className="h-5 w-5 text-muted-foreground transition-colors hover:text-foreground" />
          )}
          <span className="sr-only">切换主题</span>
        </Button>
      )}
    </header>
  )
}
