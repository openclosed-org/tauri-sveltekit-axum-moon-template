// UI Component Library — Phase 02
// Import from: $lib/components

// Primitive wrappers
export { default as Button } from './ui/Button.svelte';
export { default as Input } from './ui/Input.svelte';
export { default as Card } from './ui/Card.svelte';
export { default as Badge } from './ui/Badge.svelte';
export { default as Switch } from './ui/Switch.svelte';
export { default as Toast } from './ui/Toast.svelte';

// Compound components (re-export bits-ui sub-parts for composition)
export { default as Dialog } from './ui/Dialog.svelte';
export { Dialog as DialogParts } from 'bits-ui';

export { default as Select } from './ui/Select.svelte';
export { Select as SelectParts } from 'bits-ui';

export { default as Dropdown } from './ui/Dropdown.svelte';
export { DropdownMenu as DropdownParts } from 'bits-ui';

// Animation & Icon utilities
export { default as LottiePlayer } from './ui/LottiePlayer.svelte';
export { default as IconAnimate } from './ui/IconAnimate.svelte';
