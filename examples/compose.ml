    let compose : forall a b c. (b -> c) -> (a -> b) -> a -> c =
      fun f g x -> f (g x) in
        let inc : Int -> Int = fun x -> x + 1 in
            let is_even : Int -> Bool = fun x -> x / 2 == 0 in
                compose is_even inc 7
            end
        end
    end