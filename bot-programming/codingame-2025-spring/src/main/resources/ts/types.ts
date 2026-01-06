export type ContainerConsumer = (layer: PIXI.Container) => void

/**
 * Given by the SDK
 */
export interface FrameInfo {
  number: number
  frameDuration: number
  date: number
}
/**
 * Given by the SDK
 */
export interface CanvasInfo {
  width: number
  height: number
  oversampling: number
}
/**
 * Given by the SDK
 */
export interface PlayerInfo {
  name: string
  avatar: PIXI.Texture
  color: number
  index: number
  isMe: boolean
  number: number
  type?: string
}

export interface PlayerDto {
  message: string
}


export interface GlobalAgentDto {
  id: number
  x: number
  y: number
  cooldown: number
  optimalRange: number
  soakingPower: number
  owner: number
  balloons: number
  initialWetness: number
}

export interface DamageDto {
  t: number
  value: number
  text: string
}
export interface AgentDto extends GlobalAgentDto {
  damageHistory: DamageDto[]
  wetness: number
  shootingId: number
  hunkered: boolean
  throwingTo: CoordDto
  canShootIn: number
}
export interface FrameDataDTO {
  events: EventDto[]
  edges: CoordDto[][]
  scores: number[]
  messages: MessageDto[]
}

export interface CoordDto {
  x: number
  y: number
}

export interface AnimData {
  start: number
  end: number
}

export interface EventDto {
  type: number
  animData: AnimData
  coord: CoordDto
  target: CoordDto
  id: number
  params: number[]
}
export interface FrameData extends FrameDataDTO {
  previous: FrameData
  frameInfo: FrameInfo
  agentMap: Record<number, AgentDto>
}

export type MessageDto = {
  agentId: number
  text: string
}

export type GlobalDataDTO = {
  leagueLevel: number
  width: number
  height: number
  tiles: number[][]
  agentsPerPlayer: number
  agents: GlobalAgentDto[]
  agentMap: Record<number, GlobalAgentDto>

  /* league 3 only */
  runAndGunCoords: CoordDto[]
}

export interface Effect {
  busy: boolean
  display: PIXI.DisplayObject
}
export interface AnimatedEffect extends Effect{
  busy: boolean
  display: PIXI.AnimatedSprite
}
export interface TEffect<T extends PIXI.DisplayObject> extends Effect {
  busy: boolean
  display: T
}

export interface GlobalData extends GlobalDataDTO {
  players: PlayerInfo[]
  playerCount: number
  tileAssetMap: Record<string, string>
}


export interface Agent extends GlobalAgentDto {
  container: PIXI.Container
  // sprite: PIXI.AnimatedSprite
  animations: {
    run: PIXI.AnimatedSprite,
    shoot: PIXI.AnimatedSprite,
    hunker: PIXI.AnimatedSprite,
    throwing: PIXI.AnimatedSprite
  }
  sprite: PIXI.Container
  spriteContainer: PIXI.Container
  wetnessMask: PIXI.Sprite
  wetnessText: PIXI.Text
  wetContainer: PIXI.Container
}
