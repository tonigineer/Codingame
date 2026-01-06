function splitLine(str) {
    return str.length === 0 ? [] : str.split(' ');
}
export function parseData(unsplit, globalData) {
    const raw = unsplit.split('|');
    let idx = 0;
    const events = [];
    const eventCount = +raw[idx++];
    for (let i = 0; i < eventCount; ++i) {
        const type = +raw[idx++];
        const start = +raw[idx++];
        const end = +raw[idx++];
        const coord = parseCoord(raw[idx++]);
        const target = parseCoord(raw[idx++]);
        const params = splitLine(raw[idx++]).map(v => +v);
        const id = params[0];
        const animData = { start, end };
        events.push({
            type,
            animData,
            coord: coord,
            target: target,
            id,
            params: params.slice(1)
        });
    }
    const control = raw[idx++].split('').map(x => +x);
    const edges = [];
    for (let pIdx = 0; pIdx < globalData.playerCount; ++pIdx) {
        const playerEdges = [];
        edges.push(playerEdges);
    }
    let cIdx = 0;
    for (let y = 0; y < globalData.height; ++y) {
        for (let x = 0; x < globalData.width; ++x) {
            const playerIdx = control[cIdx++];
            if (playerIdx !== 2) {
                const coord = { x, y };
                edges[playerIdx].push(coord);
            }
        }
    }
    const scores = [];
    for (let i = 0; i < globalData.playerCount; ++i) {
        scores.push(+raw[idx++]);
    }
    const messageCount = +raw[idx++];
    const messages = [];
    for (let i = 0; i < messageCount; i++) {
        const agentId = +raw[idx++];
        const text = raw[idx++];
        messages.push({ agentId, text });
    }
    return {
        events,
        edges,
        scores,
        messages
    };
}
export function parseGlobalData(unsplit) {
    const raw = unsplit.split('|');
    let idx = 0;
    const leagueLevel = +raw[idx++];
    const w = +raw[idx++];
    const h = +raw[idx++];
    const types = raw[idx++];
    const grid = [];
    let n = 0;
    for (let y = 0; y < h; y++) {
        const row = [];
        for (let x = 0; x < w; x++) {
            row.push(+types[n++]);
        }
        grid.push(row);
    }
    const agentCount = +raw[idx++];
    const agents = [];
    const agentMap = {};
    for (let i = 0; i < agentCount; i++) {
        const id = +raw[idx++];
        const x = +raw[idx++];
        const y = +raw[idx++];
        // const agentClass = raw[idx++]
        const cooldown = +raw[idx++];
        const optimalRange = +raw[idx++];
        const soakingPower = +raw[idx++];
        const owner = +raw[idx++];
        const balloons = +raw[idx++];
        const initialWetness = +raw[idx++];
        const agent = {
            id,
            x,
            y,
            cooldown,
            optimalRange,
            soakingPower,
            owner,
            balloons,
            initialWetness
        };
        agents.push(agent);
        agentMap[id] = agent;
    }
    let runAndGunCoords = [];
    if (leagueLevel === 3) {
        const runA = parseCoord(raw[idx++]);
        const runB = parseCoord(raw[idx++]);
        const gunA = parseCoord(raw[idx++]);
        const gunB = parseCoord(raw[idx++]);
        runAndGunCoords = [runA, runB, gunA, gunB];
    }
    return {
        leagueLevel,
        width: w,
        height: h,
        tiles: grid,
        agentsPerPlayer: agentCount / 2,
        agents,
        agentMap,
        runAndGunCoords
    };
}
function parseCoord(coord) {
    const [x, y] = coord.split(' ').map(x => +x);
    return { x, y };
}
