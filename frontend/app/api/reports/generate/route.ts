import { NextRequest } from "next/server"

const RUST = "http://localhost:7437"

export async function POST(request: NextRequest) {
  const body = await request.text()
  try {
    const res = await fetch(`${RUST}/api/reports/generate`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body,
    })

    // Pass through SSE stream directly
    return new Response(res.body, {
      status: res.status,
      headers: {
        "Content-Type": "text/event-stream",
        "Cache-Control": "no-cache",
        "X-Accel-Buffering": "no",
      },
    })
  } catch {
    return new Response(
      `data: ${JSON.stringify({ type: "error", error: "Rust backend unreachable" })}\n\n`,
      {
        status: 502,
        headers: { "Content-Type": "text/event-stream" },
      }
    )
  }
}
