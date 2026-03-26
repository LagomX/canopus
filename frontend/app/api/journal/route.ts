import { NextRequest, NextResponse } from "next/server"

const RUST = "http://localhost:7437"

export async function GET(request: NextRequest) {
  const search = request.nextUrl.searchParams.toString()
  const url = `${RUST}/api/journal${search ? `?${search}` : ""}`
  try {
    const res = await fetch(url, { cache: "no-store" })
    const data = await res.json()
    return NextResponse.json(data, { status: res.status })
  } catch {
    return NextResponse.json({ error: "Rust backend unreachable" }, { status: 502 })
  }
}

export async function POST(request: NextRequest) {
  const body = await request.text()
  try {
    const res = await fetch(`${RUST}/api/journal`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body,
    })
    const data = await res.json()
    return NextResponse.json(data, { status: res.status })
  } catch {
    return NextResponse.json({ error: "Rust backend unreachable" }, { status: 502 })
  }
}
