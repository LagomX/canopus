import { NextRequest, NextResponse } from "next/server"

const RUST = "http://localhost:7437"

export async function GET() {
  try {
    const res = await fetch(`${RUST}/api/reports`, { cache: "no-store" })
    const data = await res.json()
    return NextResponse.json(data, { status: res.status })
  } catch {
    return NextResponse.json({ error: "Rust backend unreachable" }, { status: 502 })
  }
}
