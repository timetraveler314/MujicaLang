    let id : forall a. a -> a = fun x -> x in
        let apply : forall b. (b -> b) -> b -> b = fun f x -> f x in
            let z = apply id true in
                apply id (apply id 5)
            end
        end
    end