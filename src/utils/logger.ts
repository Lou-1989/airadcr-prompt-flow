/**
 * Système de logging configurable pour AirADCR Desktop
 * Permet de contrôler les logs selon l'environnement
 */

import { invoke } from '@tauri-apps/api';

export enum LogLevel {
  ERROR = 0,
  WARN = 1,
  INFO = 2,
  DEBUG = 3
}

class Logger {
  private level: LogLevel;
  private isDevelopment: boolean;
  private isTauri: boolean;

  constructor() {
    this.isDevelopment = import.meta.env.DEV;
    this.level = this.isDevelopment ? LogLevel.DEBUG : LogLevel.ERROR;
    this.isTauri = '__TAURI__' in window;
  }

  private async writeToFile(level: string, message: string, ...args: any[]) {
    if (this.isTauri) {
      const fullMessage = args.length > 0 
        ? `${message} ${JSON.stringify(args)}` 
        : message;
      
      try {
        await invoke('write_log', { 
          message: fullMessage, 
          level 
        });
      } catch (e) {
        console.error('Erreur écriture log:', e);
      }
    }
  }

  error(message: string, ...args: any[]) {
    if (this.level >= LogLevel.ERROR) {
      console.error(`[AirADCR Error] ${message}`, ...args);
      this.writeToFile('error', message, ...args);
    }
  }

  warn(message: string, ...args: any[]) {
    if (this.level >= LogLevel.WARN) {
      console.warn(`[AirADCR Warn] ${message}`, ...args);
      this.writeToFile('warn', message, ...args);
    }
  }

  info(message: string, ...args: any[]) {
    if (this.level >= LogLevel.INFO) {
      console.info(`[AirADCR Info] ${message}`, ...args);
      this.writeToFile('info', message, ...args);
    }
  }

  debug(message: string, ...args: any[]) {
    if (this.level >= LogLevel.DEBUG) {
      console.log(`[AirADCR Debug] ${message}`, ...args);
      this.writeToFile('debug', message, ...args);
    }
  }

  tauri(action: string, result?: any, error?: any) {
    if (error) {
      this.error(`Tauri ${action} failed:`, error);
    } else {
      this.debug(`Tauri ${action}:`, result);
    }
  }
}

export const logger = new Logger();