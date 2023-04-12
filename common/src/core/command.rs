use glam::Quat;

enum MoveDirection {
    Forward,
    Backward,
    Left,
    Right,
}

enum GameAction {
    Attack,
}

enum Command {
    Spawn,
    Move(MoveDirection),
    Turn(Quat),
    Action(GameAction),
}