import { IPointData } from 'pixi.js'
import { HEIGHT, WIDTH } from '../core/constants.js'
import { flagForDestructionOnReinit, getRenderer } from '../core/rendering.js'
import { bell, easeIn } from '../core/transitions.js'
import { fitAspectRatio, lerp, lerpAngle, lerpPosition, unlerp, unlerpUnclamped } from '../core/utils.js'
import { Agent, AgentDto, AnimatedEffect, AnimData, CanvasInfo, ContainerConsumer, Effect, FrameData, FrameInfo, GlobalData, PlayerInfo, TEffect } from '../types.js'
import { parseData, parseGlobalData } from './Deserializer.js'
import { initMessages, MessageContainer, renderMessageContainer } from './MessageBoxes.js'
import { TooltipManager } from './TooltipManager.js'
import { AGENT_TILE_PADDING, AVATAR_RECT, BOOM_SIZE, COVERS, DEATH_FRAMES, GAME_ZONE_RECT, GRID_LINE_WIDTH, GUN_BARREL_RATIO, GUN_TIP_RATIO, HUD_COLORS, LIQUID_TUBE_LENGTH, NAME_RECT, ORANGE_COURSE_FRAMES, ORANGE_GRENADE_FRAMES, ORANGE_PLANQUE_FRAMES, ORANGE_TIR_FRAMES, SCORE_RECT, SPLASH_ANCHOR, SPLASH_GRENADE_FRAMES, SPLASH_TIR_FRAMES, STREAM_FRAME, THROW_ANCHOR, VIOLET_COURSE_FRAMES, VIOLET_GRENADE_FRAMES, VIOLET_PLANQUE_FRAMES, VIOLET_TIR_FRAMES } from './assetConstants.js'
import ev from './events.js'
import { computeRotationAngle, normalizeAngle, rotateAround } from './trigo.js'
import { angleDiff, choice, fit, generateText, last, setAnimationProgress } from './utils.js'


//TODO: add bomb assets to agents?

// For graphic tweaking:
const COVER_SPRITE_BASE_WIDTH = 58
const WETNESS_ICON_SCALE = 0.35

interface EffectPool {
  [key: string]: Effect[]
}


const api = {
  setDebugMode: (value: boolean) => {
    api.options.debugMode = value
  },
  setWetnessIcon: (value: boolean) => {
    api.options.wetnessIcon = value
  },
  options: {
    debugMode: false,
    wetnessIcon: true,

    showOthersMessages: true,
    showMyMessages: true,
    meInGame: false,
  }
}
export { api }



export class ViewModule {
  states: FrameData[]
  globalData: GlobalData
  pool: EffectPool
  api: any
  playerSpeed: number
  previousData: FrameData
  currentData: FrameData
  progress: number
  oversampling: number
  container: PIXI.Container
  time: number
  canvasData: CanvasInfo


  messages: MessageContainer[][]
  bombLayer: PIXI.Container
  tooltipManager: TooltipManager

  gameZone: PIXI.Container
  tileSizeWithGrid: number
  tileSize: number
  agentSize: number
  tiles: {wall?: PIXI.Sprite}[]
  wallLayer: PIXI.Container
  agents: Agent[]
  waterLayer: PIXI.Container
  agentMap: Record<number, Agent>
  borders: PIXI.Graphics[]
  huds: {avatar: PIXI.Sprite, score: PIXI.Text, name: PIXI.Text}[]
  streams: PIXI.TilingSprite[]
  liquidMask: PIXI.Sprite
  fullWetnessMaskHeight: number
  hud: PIXI.Container

  constructor () {
    this.states = []
    this.pool = {}
    this.time = 0
    window.debug = this
    this.tooltipManager = new TooltipManager()
    this.api = api
    this.api.setDebugMode = (value: boolean) => {
      //hack for hiding ranking
      this.api.options.debugMode = value
      this.container.parent.children[1].visible = !value
    }
  }

  static get moduleName () {
    return 'graphics'
  }

  registerTooltip (container: PIXI.Container, getString: () => string) {
    container.interactive = true
    this.tooltipManager.register(container, getString)
  }

  // Effects
  getFromPool<T extends Effect = Effect>(type: string): T {
    if (!this.pool[type]) {
      this.pool[type] = []
    }

    for (const e of this.pool[type]) {
      if (!e.busy) {
        e.busy = true
        e.display.visible = true
        return e as T
      }
    }

    const e = this.createEffect(type)
    this.pool[type].push(e)
    e.busy = true
    return e as T
  }

  toBoardPos (coord: PIXI.IPointData) {
    return {
      x: coord.x * this.tileSizeWithGrid,
      y: coord.y * this.tileSizeWithGrid
    }
  }

  placeInGameZone(display: PIXI.DisplayObject, coord: IPointData) {
    const pos = this.toBoardPos(coord)
    display.position.set(pos.x + this.tileSizeWithGrid / 2, pos.y + this.tileSizeWithGrid / 2)
  }

  createEffect (type: string): Effect {
    let display = null
    if (type === 'bomb') {
      display = new PIXI.Container()
      const sprite = PIXI.Sprite.from('IconeGrenade')
      sprite.anchor.set(0.5)
      display.addChild(sprite)
      this.bombLayer.addChild(display)
    } else if (type === 'boom') {
      display = PIXI.AnimatedSprite.fromFrames(SPLASH_GRENADE_FRAMES)
      display.anchor.set(0.5)
      display.scale.set(fitAspectRatio(BOOM_SIZE,BOOM_SIZE, this.tileSize * 3, this.tileSize * 3))
      this.bombLayer.addChild(display)
    } else if (type === 'stream') {
      const texture = PIXI.Texture.from(STREAM_FRAME)
      display = PIXI.TilingSprite.from(STREAM_FRAME, { width: texture.width, height: texture.height })
      display.anchor.set(0, 0.5)
      this.waterLayer.addChild(display)
      this.streams.push(display)
    } else if (type === 'head_splash') {
      display = PIXI.AnimatedSprite.fromFrames(SPLASH_TIR_FRAMES)
      display.scale.set(this.tileSize / 200)
      display.anchor.copyFrom(SPLASH_ANCHOR)
      this.waterLayer.addChild(display)
      display.loop = true
      display.animationSpeed = 1
      display.play()
    } else if (type === 'death') {
      display = PIXI.AnimatedSprite.fromFrames(DEATH_FRAMES)
      display.anchor.set(0.5, 184/254)
      fit(display, this.tileSize * 2.5, this.tileSize * 2.5)
      this.wallLayer.addChild(display)
      display.loop = false
      display.animationSpeed = 0.5
    }

    return { busy: false, display }
  }

  updateScene (previousData: FrameData, currentData: FrameData, progress: number, playerSpeed?: number) {
    const frameChange = (this.currentData !== currentData)
    const fullProgressChange = ((this.progress === 1) !== (progress === 1))

    this.previousData = previousData
    this.currentData = currentData
    this.progress = progress
    this.playerSpeed = playerSpeed || 0

    this.resetEffects()

    this.updateBombs()
    this.updateAgents()
    this.updateBorders()
    this.updateHud()
    this.updateMessages()

    // Time-saving hack for hiding ranking
    this.container.parent.children[1].visible = !this.api.options.debugMode
  }

  screenshotMode() {
    this.agents.forEach(a => {
      a.wetContainer.visible = false
      if (a.owner === 1) {
        a.spriteContainer.scale.x = -1
      }
    })
    this.hud.visible = false
  }

  updateMessages () {
    // Update message
    for (const message of this.messages.flat()) {
      message.updateText('', 0, 0)
    }
    for (const messageDto of this.currentData.messages) {

      if (messageDto.text !== '') {
        const agentData = this.currentData.agentMap[messageDto.agentId]
        let agentIdx = (agentData.owner === 0 ? agentData.id : agentData.id - this.globalData.agentsPerPlayer) - 1
        const message = this.messages[agentData.owner][agentIdx]
        const agent = this.agentMap[messageDto.agentId]


        let globalPoint = this.gameZone.toGlobal(agent.container.position)
        let containerPoint = this.container.toLocal(globalPoint)
        message.updateText(messageDto.text, containerPoint.x, containerPoint.y)
      }
    }
  }

  getAnimProgress ({ start, end }: AnimData, progress: number) {
    return unlerpUnclamped(start, end, progress)
  }


  animateStreams () {
    for (const ts of this.streams) {
      ts.tilePosition.x = this.time * ts.tileScale.x
    }
  }

  updateBombs () {
    const data = this.currentData
    data.events
      .filter(e => e.type === ev.BOMB)
      .forEach(e => {
        const p = this.getAnimProgress(e.animData, this.progress)

        if (p < 0) {
          return
        }
        const throwEnd = e.params[1] / 10_000

        const throwP = unlerp(0, throwEnd, p)
        const boomP = unlerp(throwEnd, 1, p)


        if (throwP > 0 && throwP < 1) {
          const effect = this.getFromPool('bomb')
          const from = e.coord
          const to = e.target
          const pos = lerpPosition(from, to, throwP)
          this.placeInGameZone(effect.display, pos)
          effect.display.scale.set(1 + bell(throwP) * 2)
          effect.display.visible = true
        }

        if (boomP > 0) {
          const effect = this.getFromPool<AnimatedEffect>('boom')
          this.placeInGameZone(effect.display, e.target)
          effect.display.visible = true
          effect.display.alpha = 0.9

          setAnimationProgress(effect.display, boomP)
          if (this.progress === 1 && this.playerSpeed === 0) {
            setAnimationProgress(effect.display, 31/44)
            effect.display.alpha = 0.6
          }
        }
      })

  }

  updateHud () {
    const data =  this.progress === 1 ? this.currentData : this.previousData
    for (let player of this.globalData.players) {
      const {score} = this.huds[player.index]
      score.text = data.scores[player.index].toString()
      score.scale.set(1)
      this.placeInHUD(score, SCORE_RECT, player.index)
    }

    const prevScoreDiff = this.previousData.scores[1] - this.previousData.scores[0]
    const curScoreDiff = this.currentData.scores[1] - this.currentData.scores[0]
    const prevMaskWidth = lerp(-LIQUID_TUBE_LENGTH / 2, LIQUID_TUBE_LENGTH / 2, unlerp(-600, 600, prevScoreDiff))
    const curMaskWidth = lerp(-LIQUID_TUBE_LENGTH / 2, LIQUID_TUBE_LENGTH / 2, unlerp(-600, 600, curScoreDiff))
    const desiredWidth = lerp(prevMaskWidth, curMaskWidth, this.progress)

    this.liquidMask.scale.x = 1
    this.liquidMask.width = desiredWidth
  }

  updateBorders () {
    const drawData = [
      { direction: {x: -1, y: 0}, drawFrom: {x: 0, y: 0}, drawTo: {x: 0, y: 1} },
      { direction: {x: 0, y: 1}, drawFrom: {x: 0, y: 1}, drawTo: {x: 1, y: 1} },
      { direction: {x: 1, y: 0}, drawFrom: {x: 1, y: 1}, drawTo: {x: 1, y: 0} },
      { direction: {x: 0, y: -1}, drawFrom: {x: 1, y: 0}, drawTo: {x: 0, y: 0} }

    ]

    const data = this.progress < 1 ? this.previousData : this.currentData
    let allEdges = data.edges
    if (this.globalData.leagueLevel === 1) {
      allEdges = [[
        {x: 6, y: 3},
        {x: 6, y: 1}
      ], []]
    } else if (this.globalData.leagueLevel === 3) {

      allEdges = [[
        this.globalData.runAndGunCoords[0],
        this.globalData.runAndGunCoords[1],
        this.globalData.runAndGunCoords[2],
        this.globalData.runAndGunCoords[3],
      ], []]
    }

    for (let pIdx = 0; pIdx < this.globalData.playerCount; ++pIdx) {
      const border = this.borders[pIdx]
      const player = this.globalData.players[pIdx]
      const color = player.color
      border.clear()
      border.lineStyle(4, color, 1, 1)

      const zoneSet = new Set(allEdges[pIdx].map(({x, y}) => `${x},${y}`))

      const edges = allEdges[pIdx]
      for (const edge of edges) {
        const pos = this.toBoardPos(edge)
        // For each 8 directions, check if we draw a line at the perimeter of this tile
        const x = edge.x
        const y = edge.y
        for (const d of drawData) {
          const key = `${x + d.direction.x},${y + d.direction.y}`
          if (!zoneSet.has(key)) {
            const from = this.toBoardPos(d.drawFrom)
            const to = this.toBoardPos(d.drawTo)
            border.moveTo(from.x + x * this.tileSizeWithGrid, from.y + y * this.tileSizeWithGrid)
            border.lineTo(to.x + x * this.tileSizeWithGrid, to.y + y * this.tileSizeWithGrid)
          }
        }

      }
    }
  }

  upThenDown (t: number) {
    return Math.min(1, bell(t) * 2)
  }

  setAgentDirectionFromAngle(agent: Agent, angle: number): number {
    agent.spriteContainer.rotation = angle
    // if sprite upside down, flip it
    const theta = normalizeAngle(agent.spriteContainer.rotation)
    if (theta >= Math.PI / 2 && theta <= 3*Math.PI/2) {
      agent.spriteContainer.scale.x = -1
      agent.spriteContainer.rotation += Math.PI
      return -1
    } else {
      agent.spriteContainer.scale.x = 1
      return 1
    }
  }

  setAgentDirection (agent: Agent, pos: IPointData, target: IPointData) {
    const angle = Math.atan2(target.y - pos.y, target.x - pos.x)
    this.setAgentDirectionFromAngle(agent, angle)
  }

  setAgentAnimation(agent: Agent, animation: PIXI.AnimatedSprite) {
    for (const a of Object.values(agent.animations)) {
      a.visible = false
      a.stop()
    }
    animation.visible = true
    animation.play()
  }

  updateAgents () {
    const animationSet: Set<number> = new Set()

    for (const agent of this.agents) {
      const prev:AgentDto = this.previousData.agentMap[agent.id]
      const cur:AgentDto = this.currentData.agentMap[agent.id]
      agent.container.alpha = 1
      const agentDto = (cur ?? prev)
      if (agentDto != null) {
        this.placeInGameZone(agent.container, agentDto)
        agent.container.visible = true


        agent.container.zIndex = agentDto.y
        agent.container.scale.set(1)


      } else {
        agent.container.visible = false
      }
    }


    this.currentData.events
      .filter(e => e.type === ev.SHOOT)
      .forEach(e => {
        const agent = this.agentMap[e.id]
        const p = this.getAnimProgress(e.animData, this.progress)
        if (p < 0) {
          return
        }
        this.setAgentAnimation(agent, agent.animations.shoot)
        animationSet.add(agent.id)



        const gunTipOffset = {
          x: GUN_TIP_RATIO.x * this.tileSize,
          y: GUN_TIP_RATIO.y * this.tileSize
        }
        const gunBarrelOffset = {
          x: GUN_BARREL_RATIO.x * this.tileSize,
          y: GUN_BARREL_RATIO.y * this.tileSize
        }

        const stream = this.getFromPool<TEffect<PIXI.TilingSprite>>('stream')
        const from = e.coord
        const to = e.target
        const originPos = this.toBoardPos(from)
        const targetPos = this.toBoardPos(to)

        const unrotatedGunTip = {
          x: this.tileSizeWithGrid / 2 + originPos.x + gunTipOffset.x,
          y: this.tileSizeWithGrid / 2 + originPos.y + gunTipOffset.y
        }
        const unrotatedGunBarrel = {
          x: this.tileSizeWithGrid / 2 + originPos.x + gunBarrelOffset.x,
          y: this.tileSizeWithGrid / 2 + originPos.y + gunBarrelOffset.y
        }

        const agentCenter = {
          x: this.tileSizeWithGrid / 2 + originPos.x,
          y: this.tileSizeWithGrid / 2 + originPos.y
        }
        const target = {
          x: this.tileSizeWithGrid / 2 + targetPos.x,
          y: this.tileSizeWithGrid / 2 + targetPos.y
        }

        const damage = e.params[2]
        if (damage === 0) {
          // If no damage, make the target closer by one tile
          // 1) get vector from gun tip to target
          const vector = {
            x: target.x - unrotatedGunTip.x,
            y: target.y - unrotatedGunTip.y
          }
          // 2) normalize it
          const length = Math.sqrt(vector.x * vector.x + vector.y * vector.y)
          if (length > 0) {
            vector.x /= length
            vector.y /= length
          }
          // 3) move the target one tile closer
          target.x = unrotatedGunTip.x + vector.x * (length - this.tileSize)
          target.y = unrotatedGunTip.y + vector.y * (length - this.tileSize)
        }


        const agentAngle = computeRotationAngle(unrotatedGunBarrel, unrotatedGunTip, target, agentCenter)
        let newGunTip = rotateAround(unrotatedGunTip, agentCenter, agentAngle)
        let distance = Math.sqrt(Math.pow(target.x - newGunTip.x,  2) + Math.pow(target.y - newGunTip.y, 2))
        stream.display.position.set(newGunTip.x, newGunTip.y)

        let streamAngle = Math.atan2(target.y - newGunTip.y, target.x - newGunTip.x)

        const xScale = this.setAgentDirectionFromAngle(agent, agentAngle)

        if (xScale === -1) {
          newGunTip = rotateAround({
            x: this.tileSizeWithGrid / 2 + originPos.x - gunTipOffset.x,
            y: this.tileSizeWithGrid / 2 + originPos.y + gunTipOffset.y
          }, agentCenter, agentAngle + Math.PI)

          stream.display.position.set(newGunTip.x, newGunTip.y)
          streamAngle = Math.atan2(target.y - newGunTip.y, target.x - newGunTip.x)
          distance = Math.sqrt(Math.pow(target.x - newGunTip.x,  2) + Math.pow(target.y - newGunTip.y, 2))


        }



        const modifier = e.params[1]
        let alpha = modifier == 1 ? 0.6 : 0.9

        const stretchT = e.params[3] / 10_000
        const stretchP = unlerp(0, stretchT, p)
        const stayP = unlerpUnclamped(stretchT, 1, p)
        if (stayP > 0) {
          const head = this.getFromPool<AnimatedEffect>('head_splash')
          // this.placeInGameZone(head.display, to)
          head.display.position.copyFrom(target)
          head.display.rotation = streamAngle
          head.display.alpha = alpha
        }


        const width = distance * easeIn(stretchP)

        stream.display.width = width
        stream.display.rotation = streamAngle
        stream.display.alpha = alpha
        stream.display.height = this.tileSize / 9
        stream.display.tileScale.set(1, this.tileSize / 9 / stream.display.texture.height)
      })

    this.currentData.events
      .filter(e => e.type === ev.MOVE)
      .forEach(e => {
        const agent = this.agentMap[e.id]
        const p = this.getAnimProgress(e.animData, this.progress)
        if (p < 0 || p > 1) {
          return
        }

        this.setAgentAnimation(agent, agent.animations.run)
        animationSet.add(agent.id)

        const pos = lerpPosition(e.coord, e.target, p)
        this.placeInGameZone(agent.container, pos)
        this.setAgentDirection(agent, e.coord, e.target)
      })

    this.currentData.events
      .filter(e => e.type === ev.BOMB)
      .forEach(e => {
        const agent = this.agentMap[e.id]
        const p = this.getAnimProgress(e.animData, this.progress)
        if (p < 0) {
          return
        }
        this.setAgentDirection(agent, e.coord, e.target)
        this.setAgentAnimation(agent, agent.animations.throwing)
        animationSet.add(agent.id)
        const animP = unlerp(0, 0.3, p)
        setAnimationProgress(agent.animations.throwing, animP)

      })

    this.currentData.events
      .filter(e => e.type === ev.MOVE_CONFLICT)
      .forEach(e => {
        const agent = this.agentMap[e.id]
        const p = this.getAnimProgress(e.animData, this.progress)
        if (p < 0 || p > 1) {
          return
        }
        const collisionPoint = lerpPosition(e.coord, e.target, 0.2)
        const pos = lerpPosition(e.coord, collisionPoint, this.upThenDown(p))
        this.placeInGameZone(agent.container, pos)
        this.setAgentDirection(agent, e.coord, e.target)
        this.setAgentAnimation(agent, agent.animations.run)
        animationSet.add(agent.id)
      })

    this.currentData.events
      .filter(e => e.type === ev.HUNKER)
      .forEach(e => {
        const agent = this.agentMap[e.id]
        const p = this.getAnimProgress(e.animData, this.progress)
        if (p < 0) {
          return
        }
        this.setAgentAnimation(agent, agent.animations.hunker)
        animationSet.add(agent.id)
        setAnimationProgress(agent.animations.hunker, Math.min(p, 1))

      })


    this.currentData.events
      .filter(e => e.type === ev.DEATH)
      .forEach(e => {
        const agent = this.agentMap[e.id]
        const p = this.getAnimProgress(e.animData, this.progress)
        if (p < 0) {
          return
        }

        const smoke = this.getFromPool<AnimatedEffect>('death')
        this.placeInGameZone(smoke.display, e.coord)
        smoke.display.zIndex = e.coord.y + 0.5
        setAnimationProgress(smoke.display, p)

        if (p > 1) {
          agent.container.visible = false
        } else {
          agent.container.scale.set(lerp(1, 0, unlerp(0, 0.5, p)))
        }
      })


    // update wetness
    for (const agent of this.agents) {
      const prev:AgentDto = this.previousData.agentMap[agent.id]
      const cur:AgentDto = this.currentData.agentMap[agent.id]


      if (agent.container.visible) {
        let wetnessData = this.progress === 1 ? (cur ?? prev) : prev
        agent.wetnessMask.height = wetnessData.wetness/100 * this.fullWetnessMaskHeight
        agent.wetnessText.text = wetnessData.wetness.toString().padStart(2, '0')

        // No animation was set for this visible agent this frame?
        if (!animationSet.has(agent.id)) {
          this.setAgentAnimation(agent, agent.animations.run)
          agent.animations.run.gotoAndStop(0)
        }
      }
    }
  }

  resetEffects () {
    for (const type in this.pool) {
      for (const effect of this.pool[type]) {
        effect.display.visible = false
        effect.busy = false
      }
    }
  }

  animateRotation (sprite: PIXI.Sprite, rotation: number) {
    if (sprite.rotation !== rotation) {
      const eps = 0.02
      let r = lerpAngle(sprite.rotation, rotation, 0.133)
      if (angleDiff(r, rotation) < eps) {
        r = rotation
      }
      sprite.rotation = r
    }
  }

  animateScene (delta: number) {
    this.time += delta
    this.animateStreams()
    for (const player of this.globalData.players) {
      for (let i = 0; i < this.globalData.agentsPerPlayer; ++i) {
        const message = this.messages[player.index][i]
        renderMessageContainer.bind(this)(message, player.index, delta)
      }
    }
  }

  asLayer (func: ContainerConsumer): PIXI.Container {
    const layer = new PIXI.Container()
    func.bind(this)(layer)
    return layer
  }

  initAgents (layer: PIXI.Container) {
    this.agents = []
    this.agentMap = {}


    for (let globalAgent of this.globalData.agents) {
      const pIdx = globalAgent.owner
      const agentId = globalAgent.id
      const sprite = new PIXI.Container()
      const run = PIXI.AnimatedSprite.fromFrames(pIdx == 0 ? ORANGE_COURSE_FRAMES : VIOLET_COURSE_FRAMES)
      const shoot = PIXI.AnimatedSprite.fromFrames(pIdx == 0 ? ORANGE_TIR_FRAMES : VIOLET_TIR_FRAMES)
      const hunker = PIXI.AnimatedSprite.fromFrames(pIdx == 0 ? ORANGE_PLANQUE_FRAMES : VIOLET_PLANQUE_FRAMES)
      const throwing = PIXI.AnimatedSprite.fromFrames(
        (pIdx == 0 ? ORANGE_GRENADE_FRAMES : VIOLET_GRENADE_FRAMES)
      )

      const anims = [run, shoot, hunker, throwing]
      sprite.addChild(...anims)
      anims.forEach(a => {
        a.anchor.set(0.5, 0.5)
        a.visible = false
        a.animationSpeed = 1
      })
      throwing.anchor.copyFrom(THROW_ANCHOR)

      sprite.scale.set(this.tileSize / 86)

      const wetnessBackground = PIXI.Sprite.from(`${pIdx == 0 ? 'Orange' : 'Violet'}_Life_Full`)
      const wetnessMask = new PIXI.Sprite(PIXI.Texture.WHITE)
      const wetness = PIXI.Sprite.from('Life_Empty')
      const wetnessText = new PIXI.Text('00', {
        fontSize: '25px',
        fill: 0x61d2f3,
        fontWeight: 'bold'
      })
      const wetContainer = new PIXI.Container()
      ;[wetnessBackground, wetnessMask, wetness, wetnessText].forEach(s => {
        wetContainer.addChild(s)
        s.anchor.set(0.5, 1)
        fit(s, this.agentSize * WETNESS_ICON_SCALE, this.agentSize * WETNESS_ICON_SCALE)
        s.position.set(0, -this.agentSize / 2)

        Object.defineProperty(s, 'visible', {
          get: () => {
            const iconVis = this.api.options.wetnessIcon
            return s === wetnessText ? !iconVis : iconVis
          }
        })

      })
      wetness.mask = wetnessMask

      const spriteContainer = new PIXI.Container()

      const container = new PIXI.Container()
      spriteContainer.addChild(sprite)
      container.addChild(spriteContainer)
      container.addChild(wetContainer) //TODO: should be on own layer?

      layer.addChild(container)
      const agent: Agent = {
        ...globalAgent,
        container,
        spriteContainer,
        wetnessMask,
        wetnessText,
        wetContainer,
        sprite,
        animations: {
          run,
          shoot,
          hunker,
          throwing
        }
      }
      this.agents.push(agent)
      this.agentMap[agentId] = agent

      this.registerTooltip(container, () => {
        let cur = this.currentData.agentMap[agentId]
        let prev = this.previousData.agentMap[agentId]
        let data = (cur ?? prev)

        let text = `agentId: ${agentId}\n`
        +`owner: ${['Orange (0)', 'Purple (1)'][pIdx]}\n`
        +`cooldown: ${data.canShootIn} / ${globalAgent.cooldown}\n`
        +`optimal range: ${globalAgent.optimalRange}\n`
        +`soaking power: ${globalAgent.soakingPower}\n`
        if (this.progress < 1) {
          text += '---at frame end---\n'
        }
        text += `wetness: ${data.wetness}\n`
        if (cur != null) {
          if (data.hunkered) {
            text += 'hunkered down\n'
          }
          if (data.shootingId) {
            text += `shot agent ${data.shootingId}\n`
          }

          if (data.throwingTo) {
            text += `threw bomb to ${data.throwingTo.x}, ${data.throwingTo.y}\n`
          }

          if (data.damageHistory.length > 0) {
            text += 'took damage:\n'
            for (const dh of data.damageHistory) {
              text += ` ${dh.text}\n`
            }
          }
          if (data.balloons > 0) {
            text += `bombs: ${data.balloons}\n`
          }
        }
        return text.trim()
      })
    }

  }


  initGrid(layer: PIXI.Container) {
    this.tiles = []
    const gridLines = new PIXI.Graphics()
    gridLines.lineStyle(GRID_LINE_WIDTH, 0x00fff2, 1)
    gridLines.x = GRID_LINE_WIDTH
    gridLines.y = GRID_LINE_WIDTH

    for (let y = 0; y <= this.globalData.height; ++y) {
      gridLines.moveTo(0, y * this.tileSizeWithGrid)
      gridLines.lineTo(this.globalData.width * this.tileSizeWithGrid, y * this.tileSizeWithGrid)
    }
    for (let x = 0; x <= this.globalData.width; ++x) {
      gridLines.moveTo(x * this.tileSizeWithGrid, 0)
      gridLines.lineTo(x * this.tileSizeWithGrid, this.globalData.height * this.tileSizeWithGrid)
    }

    const texture = PIXI.RenderTexture.create({ width: WIDTH, height: HEIGHT})
    flagForDestructionOnReinit(texture)
    getRenderer().render(gridLines, texture)

    const gridLineSprite = new PIXI.Sprite(texture)
    gridLineSprite.alpha = 0.2
    // An offset was needed to fit the lines into the render texture, another to center it around the tiles
    gridLineSprite.position.set(-GRID_LINE_WIDTH - 1, -GRID_LINE_WIDTH - 1)
    layer.addChild(gridLineSprite)

    for (let y = 0; y < this.globalData.height; ++y) {
      for (let x = 0; x < this.globalData.width; ++x) {
        const tileType = this.globalData.tiles[y][x]
        if (tileType > 0) {
          const tileContainer = new PIXI.Container()
          tileContainer.x = this.tileSizeWithGrid * x
          tileContainer.y = this.tileSizeWithGrid * y

          const wall = PIXI.Sprite.from(this.globalData.tileAssetMap[`${x}:${y}`])

          const scale = this.tileSize / COVER_SPRITE_BASE_WIDTH
          wall.scale.set(scale)
          wall.anchor.set(0.5, 1)
          wall.position.set(this.tileSizeWithGrid / 2, this.tileSizeWithGrid)

          tileContainer.addChild(wall)
          this.wallLayer.addChild(tileContainer)
          tileContainer.zIndex = y
          this.tiles.push({wall})
        }
        this.tiles.push({})
      }
    }

    this.borders = []
    for (let pIdx = 0; pIdx < this.globalData.playerCount; ++pIdx) {
      const border = new PIXI.Graphics()
      layer.addChild(border)
      this.borders.push(border)
    }

  }


  placeInHUD(element: PIXI.Text | PIXI.Sprite, {x,y,w,h}: {x:number,y:number,w:number,h:number}, pIdx: number) {
    fit(element, w, h)
    element.position.set(pIdx ? WIDTH - 1 - x : x, y + h / 2)
    element.anchor.set(pIdx ? 1 : 0, 0.55)
  }

  initHud(layer: PIXI.Container) {
    const background = PIXI.Sprite.from('HUD.png')
    layer.addChild(background)

    this.huds = []
    for (const player of this.globalData.players) {
      const avatar = PIXI.Sprite.from(player.avatar)
      const name = new PIXI.Text(player.name, {
        fontSize: '48px',
        fill: HUD_COLORS[player.index],
        fontWeight: 'bold'
      })
      const score = new PIXI.Text('999', {
        fontSize: '48px',
        fill: HUD_COLORS[player.index],
        fontWeight: 'bold'
      })

      this.placeInHUD(avatar, AVATAR_RECT, player.index)
      this.placeInHUD(name, NAME_RECT, player.index)
      this.placeInHUD(score, SCORE_RECT, player.index)

      layer.addChild(avatar, name, score)

      this.huds.push({avatar, name, score})
    }

    // Liquid

    const scoreLiquid = PIXI.Sprite.from('HUD_Tube_Liquide.png')
    scoreLiquid.position.x = WIDTH / 2
    scoreLiquid.position.y = 46
    scoreLiquid.anchor.set(0.5, 0)
    layer.addChild(scoreLiquid)

    const mask = new PIXI.Sprite(PIXI.Texture.WHITE)
    mask.width = 0
    mask.height = 100
    mask.position.x = WIDTH / 2
    mask.position.y = 44
    scoreLiquid.mask = mask
    this.liquidMask = mask
    layer.addChild(mask)

    const glass = PIXI.Sprite.from('HUD_Tube_Dessus.png')
    glass.position.x = WIDTH / 2
    glass.position.y = 44
    glass.anchor.set(0.5, 0)
    layer.addChild(glass)



    // Company Logo

    const logoFrame = PIXI.Sprite.from('HUD_PanneauLogo.png')
    logoFrame.position.x = WIDTH / 2
    logoFrame.anchor.set(0.5, 0)
    layer.addChild(logoFrame)


    const companyLogo = PIXI.Sprite.from('codingame.png')
    companyLogo.position.x = WIDTH / 2
    companyLogo.position.y = 50
    companyLogo.anchor.set(0.5, 0.5)
    fit(companyLogo, 316, 78)
    layer.addChild(companyLogo)

  }

  reinitScene (container: PIXI.Container, canvasData: CanvasInfo) {
    (window as any).g = new PIXI.Graphics()
    this.time = 0
    this.oversampling = canvasData.oversampling
    this.container = container
    this.pool = {}
    this.canvasData = canvasData

    this.streams = []
    const tooltipLayer = this.tooltipManager.reinit()


    this.tileSizeWithGrid = Math.min(GAME_ZONE_RECT.w / this.globalData.width, GAME_ZONE_RECT.h / this.globalData.height)
    this.tileSize = this.tileSizeWithGrid - GRID_LINE_WIDTH
    this.agentSize = this.tileSize - AGENT_TILE_PADDING * 2
    this.fullWetnessMaskHeight = this.agentSize * WETNESS_ICON_SCALE

    this.wallLayer = new PIXI.Container()
    const gameZone = new PIXI.Container()
    const grid = this.asLayer(this.initGrid)
    const background = PIXI.Sprite.from('Background.jpg')
    this.hud = this.asLayer(this.initHud)
    this.bombLayer = new PIXI.Container()
    this.waterLayer = new PIXI.Container()
    this.initAgents(this.wallLayer)
    const messageLayer = this.asLayer(initMessages)

    this.wallLayer.sortableChildren = true

    gameZone.addChild(grid)
    gameZone.addChild(this.wallLayer)
    gameZone.addChild(this.waterLayer)
    gameZone.addChild(this.bombLayer)
    gameZone.addChild((window as any).g)

    gameZone.x = GAME_ZONE_RECT.x
    gameZone.y = GAME_ZONE_RECT.y
    const gameWidth = this.globalData.width * this.tileSizeWithGrid
    const gameHeight = this.globalData.height * this.tileSizeWithGrid
    gameZone.x += (GAME_ZONE_RECT.w - gameWidth) / 2
    gameZone.y += (GAME_ZONE_RECT.h - gameHeight) / 2
    this.gameZone = gameZone

    container.addChild(background)
    container.addChild(gameZone)
    container.addChild(this.hud)
    container.addChild(messageLayer)
    container.addChild(tooltipLayer)

    background.interactiveChildren = false
    this.hud.interactiveChildren = false
    tooltipLayer.interactiveChildren = false
    grid.interactiveChildren = false
    this.waterLayer.interactiveChildren = false
    this.bombLayer.interactiveChildren = false
    messageLayer.interactiveChildren = false

    container.interactive = true
    container.on('mousemove', (event) => {
      this.tooltipManager.moveTooltip(event)
    })

    this.tooltipManager.registerGlobal((data) => {
      const pos = data.getLocalPosition(gameZone)
      const x = Math.floor(pos.x / this.tileSizeWithGrid)
      const y = Math.floor(pos.y / this.tileSizeWithGrid)
      if (x < 0 || x >= this.globalData.width || y < 0 || y >= this.globalData.height) {
        return null
      }
      const blocks = []
      const tile = this.globalData.tiles[y][x]
      if (tile != null) {
        if (tile === 1) {
          blocks.push('50% cover')
        } else if (tile === 2) {
          blocks.push('75% cover')
        }
      }

      blocks.push(`(${x}, ${y})`)
      return blocks.join('\n--------\n')
    })

  }

  easeOutElastic (x: number): number {
    const c4 = (2 * Math.PI) / 3

    return x === 0
      ? 0
      : x === 1
        ? 1
        : Math.pow(2, -10 * x) * Math.sin((x * 10 - 0.75) * c4) + 1
  }

  handleGlobalData (players: PlayerInfo[], raw: string): void {
    const globalData = parseGlobalData(raw)
    api.options.meInGame = !!players.find(p => p.isMe)

    const tileAssetMap = {}
    for (let y = 0; y < globalData.height; ++y) {
      for (let x = 0; x < globalData.width; ++x) {
        const tileType = globalData.tiles[y][x]
        if (tileType > 0) {
          const asset = choice(tileType == 1 ? COVERS.METAL.LOW : COVERS.METAL.HIGH)
          tileAssetMap[`${x}:${y}`] = asset
        }
      }}

    this.globalData = {
      ...globalData,
      players: players,
      playerCount: players.length,
      tileAssetMap
    }
  }


  handleFrameData (frameInfo: FrameInfo, raw: string): FrameData {
    const dto = parseData(raw, this.globalData)
    const frameData: FrameData = {
      ...dto,
      previous: null,
      frameInfo,
      agentMap: {}

    }
    const previousFrame = last(this.states)
    if (previousFrame == null) {
      for (const globalAgent of this.globalData.agents) {
        const agent = {
          ...globalAgent,
          damageHistory: [],
          wetness: globalAgent.initialWetness,
          shootingId: null,
          hunkered: false,
          throwingTo: null,
          canShootIn: 0
        }
        frameData.agentMap[agent.id] = agent
      }
    } else {
      frameData.agentMap = {}
      for (const agentId in previousFrame.agentMap) {
        const agent = previousFrame.agentMap[agentId]
        frameData.agentMap[agentId] = {
          ...agent,
          shootingId: null,
          hunkered: false,
          throwingTo: null,
          damageHistory: [],
          canShootIn: Math.max(0, agent.canShootIn - 1)
        }
      }
    }

    frameData.previous = previousFrame ?? frameData

    for (const event of dto.events) {
      if (event.type === ev.MOVE) {
        frameData.agentMap[event.id] = {
          ...frameData.agentMap[event.id],
          x: event.target.x,
          y: event.target.y
        }
      } else if (event.type === ev.SHOOT) {
        const targetId = event.params[0]
        const target = frameData.agentMap[targetId]
        const splashHitT = lerp(event.animData.start, event.animData.end, 0.5) / frameInfo.frameDuration
        const damage = event.params[2]
        frameData.agentMap[targetId] = {
          ...target,
          wetness: target.wetness + damage,
          damageHistory: [...target.damageHistory, {
            t: splashHitT,
            value: damage,
            text: `+${damage} wetness: shot by ${event.id}`
          }]
        }
        frameData.agentMap[event.id] = {
          ...frameData.agentMap[event.id],
          shootingId: event.params[0],
          canShootIn: frameData.agentMap[event.id].cooldown
        }
      } else if (event.type === ev.DEATH) {
        delete frameData.agentMap[event.id]
      } else if (event.type === ev.HUNKER) {
        frameData.agentMap[event.id] = {
          ...frameData.agentMap[event.id],
          hunkered: true
        }
      } else if (event.type === ev.BOMB) {
        const thrower = frameData.agentMap[event.id]
        frameData.agentMap[event.id] = {
          ...thrower,
          throwingTo: event.target,
          balloons: thrower.balloons - 1
        }
        for (let affectedId of event.params.slice(2)) {
          const agent = frameData.agentMap[affectedId]
          const damage = event.params[0]
          const throwEnd = event.params[1] / 10_000
          frameData.agentMap[affectedId] = {
            ...agent,
            wetness: agent.wetness + damage,
            damageHistory: [...agent.damageHistory, {
              t: throwEnd,
              value: damage,
              text: `+${damage} wetness: bomb thrown by ${event.id}`
            }]
          }
        }
      }

      event.animData.start /= frameInfo.frameDuration
      event.animData.end /= frameInfo.frameDuration
    }

    this.states.push(frameData)
    return frameData
  }
}
