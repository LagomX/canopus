import { NextRequest, NextResponse } from "next/server"

const RUST = "http://localhost:7437"

export async function PATCH(
  request: NextRequest,
  { params }: { params: Promise<{ id: string }> }
) {
  const { id } = await params
  const body = await request.text()
  try {
    const res = await fetch(`${RUST}/api/tasks/${encodeURIComponent(id)}`, {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      body,
    })
    const data = await res.json()
    return NextResponse.json(data, { status: res.status })
  } catch {
    return NextResponse.json({ error: "Rust backend unreachable" }, { status: 502 })
  }
}

export async function DELETE(
  _request: NextRequest,
  { params }: { params: Promise<{ id: string }> }
) {
  const { id } = await params
  try {
    const res = await fetch(`${RUST}/api/tasks/${encodeURIComponent(id)}`, {
      method: "DELETE",
    })
    const data = await res.json()
    return NextResponse.json(data, { status: res.status })
  } catch {
    return NextResponse.json({ error: "Rust backend unreachable" }, { status: 502 })
  }
}
