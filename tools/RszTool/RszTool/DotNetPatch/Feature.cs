using System.ComponentModel;

#if !NET5_0_OR_GREATER
namespace System.Runtime.CompilerServices
{
    internal class IsExternalInit { }

    // <summary>
    /// Indicates that a parameter captures the expression passed for another parameter as a string.
    /// </summary>
    [AttributeUsage(AttributeTargets.Parameter, AllowMultiple = false, Inherited = false)]
    internal sealed class CallerArgumentExpressionAttribute : Attribute
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="CallerArgumentExpressionAttribute"/> class with the specified parameter name.
        /// </summary>
        /// <param name="parameterName">The name of the parameter whose expression should be captured as a string.</param>439885
        public CallerArgumentExpressionAttribute(string parameterName)
        {
            ParameterName = parameterName;
        }

        /// <summary>
        /// Gets the name of the parameter whose expression should be captured as a string.
        /// </summary>
        /// <value>
        /// The name of the parameter whose expression should be captured.
        /// </value>
        public string ParameterName { get; }
    }
}
#endif

#if !NET7_0_OR_GREATER
namespace System.Runtime.CompilerServices
{
    [AttributeUsage(AttributeTargets.Class | AttributeTargets.Struct | AttributeTargets.Field | AttributeTargets.Property, AllowMultiple = false, Inherited = false)]
    internal sealed class RequiredMemberAttribute : Attribute {}

    [AttributeUsage(AttributeTargets.All, AllowMultiple = true, Inherited = false)]
    internal sealed class CompilerFeatureRequiredAttribute : Attribute
    {
        public CompilerFeatureRequiredAttribute(string featureName)
        {
            FeatureName = featureName;
        }

        public string FeatureName { get; }
        public bool   IsOptional  { get; init; }

        public const string RefStructs      = nameof(RefStructs);
        public const string RequiredMembers = nameof(RequiredMembers);
    }
}
#endif // !NET7_0_OR_GREATER

#if !NET7_0_OR_GREATER
namespace System.Diagnostics.CodeAnalysis
{
    [AttributeUsage(AttributeTargets.Constructor, AllowMultiple = false, Inherited = false)]
    internal sealed class SetsRequiredMembersAttribute : Attribute {}
}
#endif
