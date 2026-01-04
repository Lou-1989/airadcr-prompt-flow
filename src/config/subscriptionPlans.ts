export const SUBSCRIPTION_PLANS = {
  essentiel: {
    name: 'Essentiel',
    description: 'Dictée vocale médicale professionnelle en illimité',
    features: [
      'Dictée vocale illimitée',
      'Reconnaissance vocale IA',
      'Export vers RIS/Word',
      'Support email'
    ],
    prices: {
      month: { 
        id: 'price_1Slue1Ru2nNKmcGOdNTGwuaR', 
        amount: 19,
        productId: 'prod_TjNOIbMlU3jXgj'
      },
      year: { 
        id: 'price_1Slug3Ru2nNKmcGO4QI5MzeI', 
        amount: 190, 
        savings: '17%',
        productId: 'prod_TjNQ4DTzeHIk6w'
      }
    }
  },
  pro: {
    name: 'Pro',
    description: 'Dictée vocale avec structuration IA',
    popular: true,
    features: [
      'Tout Essentiel +',
      'Structuration IA automatique',
      'Templates illimités',
      'Intégration RIS avancée',
      'Support prioritaire'
    ],
    prices: {
      month: { 
        id: 'price_1Slug9Ru2nNKmcGOC7SLg9XN', 
        amount: 39,
        productId: 'prod_TjNRFMd4UBwmFg'
      },
      year: { 
        id: 'price_1SlugARu2nNKmcGOubJJPBU7', 
        amount: 374, 
        savings: '20%',
        productId: 'prod_TjNRFadP97qYjB'
      }
    }
  },
  equipes: {
    name: 'Équipes',
    description: 'Solution sur mesure',
    features: [
      'Tout Pro +',
      'Gestion multi-utilisateurs',
      'Facturation centralisée',
      'Formation dédiée',
      'SLA personnalisé'
    ],
    contactOnly: true
  }
} as const;

export type PlanType = 'essentiel' | 'pro';
export type BillingInterval = 'month' | 'year';

export function getPriceId(plan: PlanType, interval: BillingInterval): string {
  return SUBSCRIPTION_PLANS[plan].prices[interval].id;
}

export function getPlanAmount(plan: PlanType, interval: BillingInterval): number {
  return SUBSCRIPTION_PLANS[plan].prices[interval].amount;
}
