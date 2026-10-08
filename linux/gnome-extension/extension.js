import Clutter from 'gi://Clutter';
import GLib from 'gi://GLib';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

const APP_ID = 'io.github.tangxiangong.bibcitex';
const TRAY_TITLE = 'BibCiTeX Tray';

export default class BibCiTeXTray extends Extension {
    enable() {
        this._pending = new Set();
        this._pressedIcon = null;
        // Capture only BibCiTeX's primary button, before AppIndicator's menu handler.
        this._clickSignal = global.stage.connect('captured-event', (_stage, event) => {
            const type = event.type();
            if ((type !== Clutter.EventType.BUTTON_PRESS &&
                 type !== Clutter.EventType.BUTTON_RELEASE) ||
                event.get_button() !== Clutter.BUTTON_PRIMARY)
                return Clutter.EVENT_PROPAGATE;

            const [x, y] = event.get_coords();
            const icon = this._iconAt(x, y);
            if (type === Clutter.EventType.BUTTON_PRESS) {
                this._pressedIcon = icon;
                return icon ? Clutter.EVENT_STOP : Clutter.EVENT_PROPAGATE;
            }
            const pressed = this._pressedIcon;
            this._pressedIcon = null;
            if (!pressed)
                return Clutter.EVENT_PROPAGATE;
            if (icon === pressed) {
                icon.menu?.close();
                // Let AppIndicator provide any supported activation token as usual.
                icon._indicator.open(Math.round(x), Math.round(y), event.get_time())
                    .then(() => {
                        if (!this._pending)
                            return;
                        for (const actor of global.get_window_actors()) {
                            const window = actor.meta_window;
                            if (window?.get_wm_class() === APP_ID && window.get_title() === TRAY_TITLE)
                                this._place(window);
                        }
                    })
                    .catch(error => console.error('BibCiTeX tray activation:', error));
            }
            return Clutter.EVENT_STOP;
        });
        this._mapSignal = global.window_manager.connect('map', (_manager, actor) => {
            const window = actor.meta_window;
            if (window?.get_wm_class() !== APP_ID || window.get_title() !== TRAY_TITLE)
                return;
            // Run after Shell's map handler, using the compositor's final frame size.
            const source = GLib.idle_add(GLib.PRIORITY_DEFAULT_IDLE, () => {
                this._pending.delete(source);
                if (actor.get_stage())
                    this._place(window);
                return GLib.SOURCE_REMOVE;
            });
            this._pending.add(source);
        });
    }

    _icons() {
        return Object.values(Main.panel.statusArea).filter(icon =>
            icon.visible && icon._indicator?.id === APP_ID);
    }

    _iconAt(x, y) {
        return this._icons().find(icon => {
            const [left, top] = icon.get_transformed_position();
            const [width, height] = icon.get_transformed_size();
            return x >= left && x < left + width && y >= top && y < top + height;
        }) ?? null;
    }

    _place(window) {
        const icon = this._icons()[0];
        if (!icon)
            return;
        const monitor = Main.layoutManager.findMonitorForActor(icon);
        if (!monitor)
            return;
        const [left, top] = icon.get_transformed_position();
        const [iconWidth, iconHeight] = icon.get_transformed_size();
        window.move_to_monitor(monitor.index);
        const work = window.get_work_area_for_monitor(monitor.index);
        const frame = window.get_frame_rect();
        const width = Math.min(frame.width, work.width);
        const height = Math.min(frame.height, work.height);
        const x = Math.max(work.x, Math.min(left + (iconWidth - width) / 2,
            work.x + work.width - width));
        const below = top + iconHeight + 8;
        const above = top - height - 8;
        const y = Math.max(work.y, Math.min(
            below + height <= work.y + work.height ? below : above,
            work.y + work.height - height));
        window.move_resize_frame(true, Math.round(x), Math.round(y), width, height);
        Main.activateWindow(window);
    }

    disable() {
        global.stage.disconnect(this._clickSignal);
        global.window_manager.disconnect(this._mapSignal);
        for (const source of this._pending)
            GLib.Source.remove(source);
        this._pending.clear();
        this._pending = null;
        this._pressedIcon = null;
        this._clickSignal = 0;
        this._mapSignal = 0;
    }
}
