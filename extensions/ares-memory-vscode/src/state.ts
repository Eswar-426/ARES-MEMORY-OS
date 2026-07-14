export enum AresState {
    UNINITIALIZED = 'UNINITIALIZED',
    CHECKING = 'CHECKING',
    AWAITING_CONSENT = 'AWAITING_CONSENT',
    READY = 'READY',
    DISMISSED = 'DISMISSED',
    INGESTING = 'INGESTING',
}

let currentState: AresState = AresState.UNINITIALIZED;

export function getState(): AresState {
    return currentState;
}

export function setState(newState: AresState): void {
    const oldState = currentState;
    currentState = newState;
    console.log(`[ARES] State: ${oldState} → ${newState}`);
}
