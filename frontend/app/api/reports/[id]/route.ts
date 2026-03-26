import { NextRequest, NextResponse } from "next/server"

const RUST = "http://localhost:7437"

export async function GET(_req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  const { id } = await params
  try {
    const res = await fetch(`${RUST}/api/reports/${encodeURIComponent(id)}`, { cache: "no-store" })
    const data = await res.json()
    return NextResponse.json(data, { status: res.status })
  } catch {
    return NextResponse.json({ error: "Rust backend unreachable" }, { status: 502 })
  }
}

export async function DELETE(_req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  const { id } = await params
  try {
    const res = await fetch(`${RUST}/api/reports/${encodeURIComponent(id)}`, { method: "DELETE" })
    if (res.status === 204) return new NextResponse(null, { status: 204 })
    const data = await res.json()
    return NextResponse.json(data, { status: res.status })
  } catch {
    return NextResponse.json({ error: "Rust backend unreachable" }, { status: 502 })
  }
}
