/* run.rs
 *
 * Holds the run method for Drw,
 *   as well as keypress handling
 */

use x11::xlib::{XRaiseWindow, XmbLookupString, VisibilityUnobscured, VisibilityNotify,
		SelectionNotify, DestroyNotify, FocusIn, Expose,
		XEvent, XKeyEvent, XFilterEvent, XNextEvent, KeySym, KeyPress,
		Mod1Mask, ControlMask, XLookupChars, XLookupKeySym, XLookupBoth};
use libc::iscntrl;
use std::mem::MaybeUninit;
use crate::util::grabfocus;
use crate::drw::Drw;

#[allow(non_upper_case_globals)]
impl Drw {
    pub fn run(&mut self) {
	unsafe{
	    let mut ev: XEvent = MaybeUninit::uninit().assume_init();
	    while XNextEvent(self.dpy, &mut ev) == 0 {
		if XFilterEvent(&mut ev, self.pseudo_globals.win) != 0 {
		    continue;
		}

		match ev.type_ {
		    DestroyNotify => {
			if ev.destroy_window.window != self.pseudo_globals.win {
			    break;
			}
			panic!("TODO: impliment a graceful exit");
		    },
		    Expose => {
			if ev.expose.count == 0 {
			    self.map(self.pseudo_globals.win, 0, 0, self.w, self.h);
			}
		    },
		    FocusIn => {
			/* regrab focus from parent window */
			//if ev.xfocus.window != self.pseudo_globals.win { TODO
			    grabfocus(self);
			//}
		    },
		    KeyPress => {
			if self.keypress(ev.key) {
			    break;
			}
		    },
		    SelectionNotify => {
			//if (ev.xselection.property == utf8) {
			    //paste(); // TODO
			//}
		    },
		    VisibilityNotify => {
			if ev.visibility.state != VisibilityUnobscured {
			    XRaiseWindow(self.dpy, self.pseudo_globals.win);
			}
		    },
		    _ => {},
		}
	    }
	}
    }
    
    fn keypress(&mut self, mut ev: XKeyEvent) -> bool {
	use x11::keysym::*;
	unsafe {
	    let buf: [u8; 32] = [0; 32];
	    let mut __ksym: KeySym = MaybeUninit::uninit().assume_init();
	    let mut status = MaybeUninit::uninit().assume_init();
	    let len = XmbLookupString(self.pseudo_globals.xic, &mut ev, buf.as_ptr() as *mut i8, buf.len() as i32, &mut __ksym, &mut status);
	    let mut ksym = __ksym as u32; // makes the type system shut up TODO: remove
	    match status {
		XLookupChars => {
		    if iscntrl(*(buf.as_ptr() as *mut i32)) == 0 {
			self.keyprocess(ksym, buf, len);
		    }
		},
		XLookupKeySym | XLookupBoth => {},
		_ => return false, /* XLookupNone, XBufferOverflow */
	    }
	    if (ev.state & ControlMask) != 0 {
		match ksym {
		    XK_a => ksym = XK_Home,
		    XK_b => ksym = XK_Left,
		    XK_c => ksym = XK_Escape,
		    XK_d => ksym = XK_Delete,
		    XK_e => ksym = XK_End,
		    XK_f => ksym = XK_Right,
		    XK_g | XK_bracketleft => ksym = XK_Escape,
		    XK_h => ksym = XK_BackSpace,
		    XK_i => ksym = XK_Tab,
		    XK_j | XK_J | XK_m | XK_M => {
			ksym = XK_Return;
			ev.state &= !ControlMask;
		    }
		    XK_n => ksym = XK_Down,
		    XK_p => ksym = XK_Up,
		    XK_k => {}, // delete right TODO
		    XK_u => {}, // delete left TODO
		    XK_w => {}, // delete word TODO
		    XK_y | XK_Y => {}, // paste selection TODO
		    XK_Left => {}, // TODO: move left
		    XK_Right => {}, // TODO: move right
		    XK_Return | XK_KP_Enter => {},
		    _ => return false,
		}
	    } else if (ev.state & Mod1Mask) != 0 {
		match ksym {
		    XK_b => {}, // TODO: movewordedge
		    XK_f => {}, // TODO: movewordedge
		    XK_g => ksym = XK_Home,
		    XK_G => ksym = XK_End,
		    XK_h => ksym = XK_Up,
		    XK_j => ksym = XK_Next,
		    XK_k => ksym = XK_Prior,
		    XK_l => ksym = XK_Down,
		    _ => return false,
		}
	    }
	    self.keyprocess(ksym, buf, len)
	}
    }
    
    fn keyprocess(&mut self, ksym: u32, buf: [u8; 32], _len: i32) -> bool {
	use x11::keysym::*; // TODO: I think buf can hold multiple chars
	unsafe {
	    match ksym {
		XK_Escape => return true,
		XK_Return | XK_KP_Enter => {
		    if self.items.data_matches.len() > 0 { // find the current selection
			let (partition_i, partition) = {
			    let mut partition_i = self.items.curr;
			    let mut partition = 0;
			    for p in &self.items.data_matches {
				if partition_i >= p.len() {
				    partition_i -= p.len();
				    partition += 1;
				} else {
				    break;
				}
			    }
			    (partition_i, partition)
			};
			// and print
			println!("{}", (*self.items.data_matches[partition][partition_i]).text);
		    }
		    return true;
		},
		XK_Tab => {
		    if self.items.data_matches.len() > 0 { // find the current selection
			let (partition_i, partition) = {
			    let mut partition_i = self.items.curr;
			    let mut partition = 0;
			    for p in &self.items.data_matches {
				if partition_i >= p.len() {
				    partition_i -= p.len();
				    partition += 1;
				} else {
				    break;
				}
			    }
			    (partition_i, partition)
			}; // and autocomplete
			self.input = (*self.items.data_matches[partition][partition_i]).text.clone();
			self.pseudo_globals.cursor = self.input.len();			
			self.items.curr = 0;
			self.draw();
		    }
		},
		XK_Home => {
		    if self.items.data_matches.len() > 0 {
			self.items.curr = 0;
			self.draw();
		    }
		},
		XK_End => {
		    if self.items.data_matches.len() > 0 {
			self.items.curr = self.items.data_matches.iter().fold(0, |acc, cur| acc+cur.len())-1;
			self.draw();
		    }
		},
		XK_Next => { // PgDn
		    let mut partition_i = self.items.curr;
		    let mut partition = 0;
		    for p in &self.items.data_matches {
			if partition_i >= p.len() {
			    partition_i -= p.len();
			    partition += 1;
			} else {
			    break;
			}
		    }
		    if partition+1 < self.items.data_matches.len() {
			self.items.curr += self.items.data_matches[partition].len()-partition_i;
			self.draw();
		    }
		},
		XK_Prior => { // PgUp
		    let mut partition_i = self.items.curr;
		    let mut partition = 0;
		    for p in &self.items.data_matches {
			if partition_i >= p.len() {
			    partition_i -= p.len();
			    partition += 1;
			} else {
			    break;
			}
		    }
		    if partition > 0 {
			self.items.curr -= self.items.data_matches[partition-1].len()+partition_i;
			self.draw();
		    }
		},
		XK_Left => {
		    if self.pseudo_globals.cursor == self.input.len() && self.items.curr > 0 { // move selection
			    self.items.curr -= 1;
			    self.draw();
		    } else { // move cursor
			if self.pseudo_globals.cursor > 0 {
			    self.pseudo_globals.cursor -= 1;
			    self.draw();
			}
		    }
		},
		XK_Right => {
		    if self.pseudo_globals.cursor == self.input.len() { // move selection
			if self.items.curr+1 < self.items.data_matches.iter().fold(0, |acc, cur| acc+cur.len()) {
			    self.items.curr += 1;
			    self.draw();
			}
		    } else { // move cursor
			self.pseudo_globals.cursor += 1;
			self.draw();
		    }
		},
		XK_Up => {
		    if self.items.curr > 0 {
			self.items.curr -= 1;
			self.draw();
		    }
		},
		XK_Down => {
		    if self.items.curr+1 < self.items.data_matches.iter().fold(0, |acc, cur| acc+cur.len()) {
			self.items.curr += 1;
			self.draw();
		    }
		},
		XK_BackSpace => {
		    if self.pseudo_globals.cursor > 0 {
			let mut char_iter = self.input.chars();
			let mut new = String::new();
			new.push_str(&(&mut char_iter).take(self.pseudo_globals.cursor-1).collect::<String>());
			char_iter.next(); // get rid of one char
			new.push_str(&char_iter.collect::<String>());
			self.input = new;
			self.pseudo_globals.cursor -= 1;
			self.draw();
		    }
		},
		XK_Delete => {
		    if self.pseudo_globals.cursor < self.input.len() {
			let mut char_iter = self.input.chars();
			let mut new = String::new();
			new.push_str(&(&mut char_iter).take(self.pseudo_globals.cursor).collect::<String>());
			char_iter.next(); // get rid of one char
			new.push_str(&char_iter.collect::<String>());
			self.input = new;
			self.draw();
		    }
		},
		ch => { // all others, assumed to be normal chars
		    if iscntrl(*(buf.as_ptr() as *mut i32)) == 0 {
			//println!("?"); // TODO: numpad input breaks this
			let mut char_iter = self.input.chars();
			let mut new = String::new();
			new.push_str(&(&mut char_iter).take(self.pseudo_globals.cursor).collect::<String>());
			let to_push = std::char::from_u32(ch);
			if to_push.is_none() {
			    return false;
			}
			new.push(to_push.unwrap());
			new.push_str(&char_iter.collect::<String>());
			self.input = new;
			self.pseudo_globals.cursor += 1;
			self.items.curr = 0;
			self.draw();
		    }
		},
	    }
	}
	false
    }
}