use winapi::um::winuser::*;

pub fn jiggle(movement : i32  ) {
    unsafe {
        let mut input_u: INPUT_u = std::mem::zeroed();
        *input_u.mi_mut() = MOUSEINPUT {
            dx: 0,
            dy: movement,
            mouseData: 0,
            dwFlags: MOUSEEVENTF_MOVE,
            time: 0,
            dwExtraInfo: 0,
        };
        let mut input = INPUT {
            type_: INPUT_MOUSE,
            u: input_u,
        };
        let ipsize = std::mem::size_of::<INPUT>() as i32;
        SendInput(1, &mut input, ipsize);
    }
}

                              

