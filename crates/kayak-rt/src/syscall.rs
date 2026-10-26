//! Syscalls (`CLAUDES_CHOICES.md` C225, `docs/research/syscalls.md`):
//! what kayak-rt provides by name to a runtime written in KFX, at a type
//! both sides state, compared as text in canonical form, as Fixpt's
//! `fixpt_runtime::syscall` does. A KFX program declares each with
//! `define-syscall` or `define-optional-syscall`, which the checker expands
//! into calls of these (`syscall_require`, `syscall_available`,
//! `syscallN`), each given its label for messages, its host name and its
//! type's text; what crosses crosses as `Native` words, Fixpt's layout.

use crate::{Native, Str, fail};

/// One syscall: its host name, its type in canonical form, and what it
/// does with its arguments, each a word of Fixpt's layout.
struct Syscall {
    host: &'static str,
    canonical: &'static str,
    run: fn(&[u64]) -> u64,
}

fn int(w: u64) -> i64 {
    i64::of_word(w)
}

/// The syscalls kayak-rt has.
const SYSCALLS: &[Syscall] = &[
    // An executable view of a flat array's range (kayak-jit's `exec_view`):
    // words `start` to `start + n` of the array, whole pages of a
    // reservation nothing has written there; the distance in bytes from
    // the array's address to the view's, or -1.
    Syscall {
        host: "code:exec-view/1",
        canonical: "(poly ((r1 region)) (subr (maxeff (io sys) (write r1)) ((flatarrayof i64 r1) int int) int))",
        run: |a| {
            let (start, n) = (int(a[1]), int(a[2]));
            let at = (a[0] & !7) as usize;
            let view = match (usize::try_from(start), usize::try_from(n)) {
                (Ok(s), Ok(n)) => kayak_jit::exec_view(at + 8 * s, 8 * n)
                    .map(|rx| rx as i64 - (at + 8 * s) as i64),
                _ => None,
            };
            view.unwrap_or(-1).word()
        },
    },
    Syscall {
        host: "code:page-words/1",
        canonical: "(subr (io sys) () int)",
        run: |_| ((kayak_jit::page_bytes() / 8) as i64).word(),
    },
    // The instruction cache made to see words `start` to `start + n` of a
    // viewed range as written: none, where no view holds them.
    Syscall {
        host: "code:flush/1",
        canonical: "(poly ((r1 region)) (subr (maxeff (io sys) (read r1)) ((flatarrayof i64 r1) int int) unit))",
        run: |a| {
            let (start, n) = (int(a[1]), int(a[2]));
            if let (Ok(s), Ok(n)) = (usize::try_from(start), usize::try_from(n)) {
                kayak_jit::flush_written((a[0] & !7) as usize + 8 * s, 8 * n);
            }
            ().word()
        },
    },
    // A call of native code a runtime made (tier 1's code area): `m` a
    // mailbox of words, the closure, the entry's address, then `n`
    // arguments, each a word of Fixpt's layout; the answer put in `m[0]`.
    // 0, or why not: a trap's kind (1 to 5), 6 a routine kayak-jit has not,
    // 7 a trap said otherwise (its words not kept).
    Syscall {
        host: "kayak:call-native/1",
        canonical: "(poly ((r1 region)) (subr (maxeff (io sys) (read r1) (write r1)) ((flatarrayof i64 r1) int) int))",
        run: |a| {
            kayak_jit::call_mailbox((a[0] & !7) as usize, int(a[1]).clamp(0, 8) as usize).word()
        },
    },
];

/// What kayak-rt has under `host`: the syscall, if its type is `text`;
/// else its own type, if it has the name at another.
fn find(host: &str, text: &str) -> Result<&'static Syscall, Option<&'static str>> {
    match SYSCALLS.iter().find(|s| s.host == host) {
        Some(s) if s.canonical == text => Ok(s),
        Some(s) => Err(Some(s.canonical)),
        None => Err(None),
    }
}

/// A required syscall's check, as its declaration's definition is made.
pub fn syscall_require(label: Str, host: Str, text: Str) {
    match find(host.as_str(), text.as_str()) {
        Ok(_) => {}
        Err(None) => fail(&format!("{}: this runtime has none", label.as_str())),
        Err(Some(theirs)) => fail(&format!(
            "{}: declared {}; this runtime's is {theirs}",
            label.as_str(),
            text.as_str()
        )),
    }
}

/// Whether kayak-rt has `host` at the type `text`.
pub fn syscall_available(host: Str, text: Str) -> bool {
    find(host.as_str(), text.as_str()).is_ok()
}

fn call(label: Str, host: Str, text: Str, args: &[u64]) -> u64 {
    match find(host.as_str(), text.as_str()) {
        Ok(s) => (s.run)(args),
        Err(_) => fail(&format!(
            "{}: not available on this runtime",
            label.as_str()
        )),
    }
}

macro_rules! syscall_n {
    ($name:ident; $($a:ident $x:ident),*) => {
        /// A call of a syscall of as many arguments, given the declaration's
        /// label, host name and type's text.
        #[allow(clippy::too_many_arguments)]
        pub fn $name<$($a: Native,)* R: Native>(label: Str, host: Str, text: Str $(, $x: $a)*) -> R {
            R::of_word(call(label, host, text, &[$($x.word()),*]))
        }
    };
}

syscall_n!(syscall0;);
syscall_n!(syscall1; A a);
syscall_n!(syscall2; A a, B b);
syscall_n!(syscall3; A a, B b, C c);
syscall_n!(syscall4; A a, B b, C c, D d);
syscall_n!(syscall5; A a, B b, C c, D d, E e);
syscall_n!(syscall6; A a, B b, C c, D d, E e, F f);
