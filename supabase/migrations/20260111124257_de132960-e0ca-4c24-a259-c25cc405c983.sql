-- Add DELETE policy for customers table
CREATE POLICY "Users can delete their own customer data" 
ON public.customers 
FOR DELETE 
USING (auth.uid() = id);

-- Add DELETE policy for subscriptions table
CREATE POLICY "Users can delete their own subscriptions" 
ON public.subscriptions 
FOR DELETE 
USING (auth.uid() = user_id);