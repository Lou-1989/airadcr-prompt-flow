/**
 * Système de logging configurable pour AirADCR Desktop
 * Permet de contrôler les logs selon l'environnement
 */

export enum LogLevel {
  ERROR = 0,
  WARN = 1,
  INFO = 2,
  DEBUG = 3
}

class Logger {
  private level: LogLevel;
  private isDevelopment: boolean;

  constructor() {
    this.isDevelopment = import.meta.env.DEV;
    this.level = this.isDevelopment ? LogLevel.DEBUG : LogLevel.ERROR;
  }

  error(message: string, ...args: any[]) {
    if (this.level >= LogLevel.ERROR) {
      console.error(`[AirADCR Error] ${message}`, ...args);
    }
  }

  warn(message: string, ...args: any[]) {
    if (this.level >= LogLevel.WARN) {
      console.warn(`[AirADCR Warn] ${message}`, ...args);
    }
  }

  info(message: string, ...args: any[]) {
    if (this.level >= LogLevel.INFO) {
      console.info(`[AirADCR Info] ${message}`, ...args);
    }
  }

  debug(message: string, ...args: any[]) {
    if (this.level >= LogLevel.DEBUG) {
      console.log(`[AirADCR Debug] ${message}`, ...args);
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