import { Application, Router } from "https://deno.land/x/oak@v11.1.0/mod.ts";

/// Mock Data
interface HistoryEntry {
    timestamp: string,
    killer_name: string,
    killer_ship: string,
    killer_rank: string,
    victim_name: string,
    victim_ship: string,
    victim_rank: string,
    location: string
}

interface CmdrWhoisLookupResponseSuccess {
    cmdrName: string,
    kills: number // u_int
    deaths: number // u_int
    recentHistory: Required<HistoryEntry>[] // Like the last 5 or so
                                  // entries
}

const data= {
    "success": "User data returned",
    "cmdrName": "Harry Potter",
    "kills": 3,
    "deaths": 0,
    "recentHistory": [
        {
            "id": 4,
            "timestamp": "2007-06-30 18:50:22",
            "killer_name": "Harry Potter",
            "killer_rank": "9",
            "killer_ship": "Mamba",
            "victim_name": "Salomé",
            "victim_rank": "3",
            "victim_ship": "Unknown",
            "created_at": null,
            "updated_at": null,
            "location": "Unknown"
        },
        {
            "id": 4290,
            "timestamp": "2018-03-08 20:38:08",
            "killer_name": "Harry Potter",
            "killer_rank": "8",
            "killer_ship": "empire_trader",
            "victim_name": "ST4R F0X",
            "victim_rank": "4",
            "victim_ship": "Federation_Corvette",
            "created_at": null,
            "updated_at": null,
            "location": "Unknown"
        },
        {
            "id": 4392,
            "timestamp": "2019-03-17 18:01:12",
            "killer_name": "Harry Potter",
            "killer_rank": "8",
            "killer_ship": "ferdelance",
            "victim_name": "ST4R F0X",
            "victim_rank": "6",
            "victim_ship": "Empire_Trader",
            "created_at": null,
            "updated_at": null,
            "location": "Unknown"
        }
    ]
}




///






const app = new Application()

const router = new Router();

router.get("/api/bot/user/:userId", (ctx) => {
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