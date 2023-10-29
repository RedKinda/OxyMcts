use core::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::ops::{Add, Div};
use std::sync::Mutex;

use ascii_tree::Tree::{Leaf, Node};
use ascii_tree::{write_tree, Tree};
use num_traits::{ToPrimitive, Zero};

use crate::aliases::{LazyMctsNode, LazyMctsTree};
use crate::traits::{BackPropPolicy, GameTrait, LazyTreePolicy, Playout};
use crate::tree::NodeId;
use crate::Evaluator;

/// This is a special MCTS because it doesn't store the state in the node but instead stores the
/// historic to the node.

pub struct LazyMcts<State, TP, PP, BP, EV, AddInfo, Reward>
where
    State: GameTrait,
    TP: LazyTreePolicy<State, EV, AddInfo, Reward>,
    PP: Playout<State>,
    BP: BackPropPolicy<Vec<State::Move>, State::Move, Reward, AddInfo, EV::EvalResult>,
    EV: Evaluator<State, Reward, AddInfo>,
    AddInfo: Clone + Default,
    Reward: Clone,
{
    root_state: State,
    tree_policy: PhantomData<TP>,
    playout_policy: PhantomData<PP>,
    backprop_policy: PhantomData<BP>,
    evaluator: PhantomData<EV>,
    tree: LazyMctsTree<State, Reward, AddInfo>,
}

impl<State, TP, PP, BP, EV, A, R> LazyMcts<State, TP, PP, BP, EV, A, R>
where
    State: GameTrait,
    TP: LazyTreePolicy<State, EV, A, R>,
    PP: Playout<State>,
    BP: BackPropPolicy<Vec<State::Move>, State::Move, R, A, EV::EvalResult>,
    EV: Evaluator<State, R, A>,
    A: Clone + Default,
    R: Clone + Div + ToPrimitive + Zero + Add + Display,
{
    pub fn new(root_state: State) -> Self {
        Self::with_capacity(root_state, 0)
    }

    pub fn with_capacity(root_state: State, capacity: usize) -> Self {
        let tree = LazyMctsTree::<State, R, A>::with_capacity(
            LazyMctsNode::<State, R, A> {
                sum_rewards: Zero::zero(),
                n_visits: 0,
                unvisited_moves: root_state.legals_moves(),
                hash: root_state.hash(),
                state: vec![],
                additional_info: Default::default(),
            },
            capacity,
        );
        Self {
            root_state,
            tree_policy: PhantomData,
            playout_policy: PhantomData,
            backprop_policy: PhantomData,
            evaluator: PhantomData,
            tree,
        }
    }

    /// Executes one selection, expansion?, simulation, backpropagation.
    pub fn execute(&self, evaluation_args: &EV::Args, playout_args: PP::Args) {
        let (node_id, state) =
            TP::tree_policy(&self.tree, self.root_state.clone(), evaluation_args);

        let final_state = PP::playout(state, playout_args);
        let eval = EV::evaluate_leaf(final_state, &self.root_state.player_turn());

        BP::backprop(&self.tree, node_id, eval);
    }

    /// Returns the best move from the root.
    pub fn best_move(&self, evaluator_args: &EV::Args) -> State::Move {
        let best_child = TP::best_child(
            &self.tree,
            &self.root_state.player_turn(),
            self.tree.root_id(),
            evaluator_args,
        );
        self.tree
            .get(best_child)
            .unwrap()
            .value()
            .state
            .last()
            .expect("The historic of the children of the root is empty, cannot happen")
            .clone()
    }
}

// impl<State, TP, PP, BP, EV, A, R> Debug for LazyMcts<State, TP, PP, BP, EV, A, R>
// where
//     State: GameTrait,
//     TP: LazyTreePolicy<State, EV, A, R>,
//     PP: Playout<State>,
//     BP: BackPropPolicy<Vec<State::Move>, State::Move, R, A, EV::EvalResult>,
//     EV: Evaluator<State, R, A>,
//     EV::EvalResult: Debug,
//     A: Clone + Default + Debug,
//     R: Clone + Debug + Div + Add + Zero + ToPrimitive,
// {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         f.write_str(&format!("{:?}", self.tree))
//     }
// }

impl<State, TP, PP, BP, EV, A, R> Clone for LazyMcts<State, TP, PP, BP, EV, A, R>
where
    State: GameTrait,
    TP: LazyTreePolicy<State, EV, A, R>,
    PP: Playout<State>,
    BP: BackPropPolicy<Vec<State::Move>, State::Move, R, A, EV::EvalResult>,
    EV: Evaluator<State, R, A>,
    A: Clone + Default,
    R: Clone + Debug + Div + Add + Zero + ToPrimitive,
{
    fn clone(&self) -> Self {
        Self {
            root_state: self.root_state.clone(),
            tree_policy: PhantomData,
            playout_policy: PhantomData,
            backprop_policy: PhantomData,
            evaluator: PhantomData,
            tree: self.tree.clone(),
        }
    }
}
