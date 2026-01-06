import { EndScreenModule } from './endscreen-module/EndScreenModule.js'
import { ViewModule, api } from './graphics/ViewModule.js'

export const modules = [
  ViewModule,
  EndScreenModule
]

export const playerColors = [
  '#b23200', // fire red
  '#681d79', // jam purple
]

export const gameName = 'Summer2025'

export const stepByStepAnimateSpeed = 3

export const options = [
  {
    title: 'HIDE RANKING',
    get: function () {
      return api.options.debugMode
    },
    set: function (value) {
      api.setDebugMode(value)
    },
    values: {
      'ON': true,
      'OFF': false
    },
  }, {
    title: 'WETNESS',
    get: function () {
      return api.options.wetnessIcon
    },
    set: function (value) {
      api.setWetnessIcon(value)
    },
    values: {
      'ICON': true,
      'NUMBER': false
    },
  }, {
    title: 'MY MESSAGES',
    get: function () {
      return api.options.showMyMessages
    },
    set: function (value) {
      api.options.showMyMessages = value
    },
    enabled: function () {
      return api.options.meInGame
    },
    values: {
      'ON': true,
      'OFF': false
    }
  }, {
    title: 'OTHERS\' MESSAGES',
    get: function () {
      return api.options.showOthersMessages
    },
    set: function (value) {
      api.options.showOthersMessages = value
    },

    values: {
      'ON': true,
      'OFF': false
    }
  }
]
