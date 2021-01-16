mod tax;

fn main() {
    let income = 80000;
    let provincial_taxes = tax::compute_provincial(income, 2020);
    let federal_taxes = tax::compute_federal(income, 2020);
    let taxes = provincial_taxes + federal_taxes;
    let net_income = income - taxes;
    println!("{}", net_income);
}
