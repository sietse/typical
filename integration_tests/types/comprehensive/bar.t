import 'main.t'

choice Bar {
    a_required = 0
    b_required: F64 = 1
    c_required: U64 = 2
    d_required: S64 = 3
    e_required: Bool = 4
    f_required: Bytes = 5
    g_required: String = 6
    h_required: [Unit] = 7
    i_required: [F64] = 8
    j_required: [U64] = 9
    k_required: [S64] = 10
    l_required: [Bool] = 11
    m_required: [Bytes] = 12
    n_required: [String] = 13
    o_required: [[main.EmptyStruct]] = 14
    asymmetric a_asymmetric = 16
    asymmetric b_asymmetric: F64 = 17
    asymmetric c_asymmetric: U64 = 18
    asymmetric d_asymmetric: S64 = 19
    asymmetric e_asymmetric: Bool = 20
    asymmetric f_asymmetric: Bytes = 21
    asymmetric g_asymmetric: String = 22
    asymmetric h_asymmetric: [Unit] = 23
    asymmetric i_asymmetric: [F64] = 24
    asymmetric j_asymmetric: [U64] = 25
    asymmetric k_asymmetric: [S64] = 26
    asymmetric l_asymmetric: [Bool] = 27
    asymmetric m_asymmetric: [Bytes] = 28
    asymmetric n_asymmetric: [String] = 29
    asymmetric o_asymmetric: [[main.EmptyStruct]] = 30
    optional a_optional = 32
    optional b_optional: F64 = 33
    optional c_optional: U64 = 34
    optional d_optional: S64 = 35
    optional e_optional: Bool = 36
    optional f_optional: Bytes = 37
    optional g_optional: String = 38
    optional h_optional: [Unit] = 39
    optional i_optional: [F64] = 40
    optional j_optional: [U64] = 41
    optional k_optional: [S64] = 42
    optional l_optional: [Bool] = 43
    optional m_optional: [Bytes] = 44
    optional n_optional: [String] = 45
    optional o_optional: [[main.EmptyStruct]] = 46

    deleted 15 31 47
}
