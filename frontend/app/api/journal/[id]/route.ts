import { NextRequest, NextResponse } from "next/server"

const RUST = "http://localhost:7437"

export async function DELETE(
  _request: NextRequest,
  { params }: { params: Promise<{ id: string }> }
) {
  const { id } = await params
  try {
    const res = await fetch(`${RUST}/api/journal/${encodeURIComponent(id)}`, {
      method: "DELETE",
    })
    const data = await res.json()
    return NextResponse.json(data, { status: res.status })
  } catch {
    return NextResponse.json({ error: "Rust backend unreachable" }, { status: 502 })
  }
}
