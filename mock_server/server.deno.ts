import { Application, Router } from "https://deno.land/x/oak@v11.1.0/mod.ts";

/// Mock Data
interface HistoryEntry {
    eventType: "died" | "killed"
    otherCmdr: string // Killer or Victim, depending on Event
    timestamp: number // unix time (could also be iso8601 /shrug)
}

interface CmdrWhoisLookupResponseSuccess {
    cmdrName: string,
    kills: number // u_int
    deaths: number // u_int
    recentHistory: HistoryEntry[] // Like the last 5 or so
                                  // entries
}

const data: CmdrWhoisLookupResponseSuccess = {
    cmdrName: "MockName",
    kills: 43,
    deaths: 21,
    recentHistory: [
        {
            eventType: "killed",
            otherCmdr: "WDX",
            timestamp: 1671647068
        },
        {
            eventType: "killed",
            otherCmdr: "Banana",
            timestamp: 1671637068
        },
        {
            eventType: "died",
            otherCmdr: "Egg",
            timestamp: 1671627068
        }
    ]
}



///






const app = new Application()

const router = new Router();

router.get("/api/user/:userId", (ctx) => {
    console.log(ctx.request.headers)
    const {userId} = ctx.params
    if (userId !== "WDX") {
        ctx.response.status = 404;
        ctx.response.body = {}
        
    } else {
        ctx.response.status = 200
        ctx.response.body = data
    }
})


app.use(router.routes())
app.use(router.allowedMethods())

await app.listen({port: 80})